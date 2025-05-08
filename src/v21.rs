use std::{f32::consts::PI, ops::Rem};
use std::collections::VecDeque;
use fundsp::hacker32::U1;
use fundsp::audionode::AudioNode;
use fundsp::prelude::Frame;

pub struct V21RX {
    sampling_period: f32,
    samples_per_symbol: usize,
    omega1: f32,
    omega0: f32,

    // Atributos auxiliares
    sample_buffer: VecDeque<f32>,
    v0r_buffer: f32,
    v0i_buffer: f32,
    v1r_buffer: f32,
    v1i_buffer: f32,
    raw_decision_buffer: [f32; 2],
    filtered_decision_buffer: [f32; 2],
    lowpass: fundsp::filter::ButterLowpass<f32, f32, U1>,
    low_difference_counter: usize,
    state: State,
}

enum State {
    Idle,
    CarrierDetected,
}

impl V21RX {
    pub fn new(
        sampling_period: f32,
        samples_per_symbol: usize,
        omega1: f32,
        omega0: f32,
    ) -> Self {
        let l = samples_per_symbol as f32;

        Self {
            sampling_period,
            samples_per_symbol,
            omega1,
            omega0,
            sample_buffer: VecDeque::from(vec![0.0; samples_per_symbol + 1]),
            v0r_buffer: 0.0,
            v0i_buffer: 0.0,
            v1r_buffer: 0.0,
            v1i_buffer: 0.0,
            raw_decision_buffer: [0.0; 2],
            filtered_decision_buffer: [0.0; 2],
            lowpass: fundsp::filter::ButterLowpass::new(300.),
            low_difference_counter: 0,
            state: State::Idle,
        }
    }

    pub fn demodulate(&mut self, in_samples: &[f32], out_samples: &mut [u8]) {
        let l = self.samples_per_symbol;
        self.lowpass.set_sample_rate(1. / self.sampling_period as f64);

        for (i, &sample) in in_samples.iter().enumerate() {
            self.sample_buffer.push_front(sample);

            let v0r = self.sample_buffer[0]
                - 0.99f32.powi(self.samples_per_symbol as i32) * (self.omega0*self.samples_per_symbol as f32*self.sampling_period).cos() * self.sample_buffer[l]
                + 0.99*(self.omega0*self.sampling_period).cos() * self.v0r_buffer
                - 0.99*(self.omega0*self.sampling_period).sin() * self.v0i_buffer;

            let v0i = -0.99f32.powi(self.samples_per_symbol as i32) * (self.omega0*self.samples_per_symbol as f32*self.sampling_period).sin() * self.sample_buffer[l]
                + 0.99*(self.omega0*self.sampling_period).sin() * self.v0r_buffer
                + 0.99*(self.omega0*self.sampling_period).cos() * self.v0i_buffer;                

            let v1r = self.sample_buffer[0]
                - 0.99f32.powi(self.samples_per_symbol as i32) * (self.omega1*self.samples_per_symbol as f32*self.sampling_period).cos() * self.sample_buffer[l]
                + 0.99*(self.omega1*self.sampling_period).cos() * self.v1r_buffer
                - 0.99*(self.omega1*self.sampling_period).sin() * self.v1i_buffer;

            let v1i = -0.99f32.powi(self.samples_per_symbol as i32) * (self.omega1*self.samples_per_symbol as f32*self.sampling_period).sin() * self.sample_buffer[l]
                + 0.99*(self.omega1*self.sampling_period).sin() * self.v1r_buffer
                + 0.99*(self.omega1*self.sampling_period).cos() * self.v1i_buffer;

            let raw_decision = v1r * v1r + v1i * v1i - v0r * v0r - v0i * v0i;

            let filtered_decision = *self.lowpass.tick(&Frame::from([raw_decision])).first().unwrap();

            out_samples[i] = match self.state {
                State::Idle => {
                    if filtered_decision.abs() > 120.0 {
                        self.low_difference_counter = 0;
                        self.state = State::CarrierDetected;
                        if filtered_decision > 0.0 { 1 } else { 0 }
                    } else {
                        1
                    }
                }
                State::CarrierDetected => {
                    if filtered_decision.abs() < 60.0 {
                        self.low_difference_counter += 1;
                    } else {
                        self.low_difference_counter = 0;
                    }

                    if self.low_difference_counter >= 50 {
                        self.state = State::Idle;
                        1
                    } else {
                        if filtered_decision > 0.0 { 1 } else { 0 }
                    }
                }
            };

            self.sample_buffer.pop_back();
            self.v0r_buffer = v0r;
            self.v0i_buffer = v0i;
            self.v1r_buffer = v1r;
            self.v1i_buffer = v1i;
            self.raw_decision_buffer[1] = self.raw_decision_buffer[0];
            self.raw_decision_buffer[0] = raw_decision;
            self.filtered_decision_buffer[1] = self.filtered_decision_buffer[0];
            self.filtered_decision_buffer[0] = filtered_decision;
        }
    }
}

pub struct V21TX {
    sampling_period: f32,
    omega1: f32,
    omega0: f32,
    phase: f32,
}

impl V21TX {
    pub fn new(sampling_period: f32, omega1: f32, omega0: f32) -> Self {
        Self {
            sampling_period,
            omega1,
            omega0,
            phase: 0.,
        }
    }

    pub fn modulate(&mut self, in_samples: &[u8], out_samples: &mut [f32]) {
        debug_assert!(in_samples.len() == out_samples.len());

        for i in 0..in_samples.len() {
            out_samples[i] = self.phase.sin();

            let omega = if in_samples[i] == 0 {
                self.omega0
            } else {
                self.omega1
            };
            self.phase = (self.phase + self.sampling_period * omega).rem(2. * PI);
        }
    }
}
