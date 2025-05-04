use std::{f32::consts::PI, ops::Rem};
use std::collections::VecDeque;

pub struct V21RX {
    // TODO: coloque outros atributos que você precisar aqui
    sampling_period: f32,
    samples_per_symbol: usize,
    omega_mark: f32,
    omega_space: f32,
    r: f32,
    v0r_last: f32,
    v0i_last: f32,
    v1r_last: f32,
    v1i_last: f32,
    history: VecDeque<f32>,
    out_index: usize,
}

impl V21RX {
    pub fn new(
        sampling_period: f32,
        samples_per_symbol: usize,
        omega_mark: f32,
        omega_space: f32,
    ) -> Self {
        // TODO: inicialize seus novos atributos abaixo
        Self {
            sampling_period,
            samples_per_symbol,
            omega_mark,
            omega_space,
            r: 0.999,
            v0r_last: 0.0,
            v0i_last: 0.0,
            v1r_last: 0.0,
            v1i_last: 0.0,
            history: VecDeque::with_capacity(samples_per_symbol),
            out_index: 0,
        }
    }

    pub fn demodulate(&mut self, in_samples: &[f32], out_samples: &mut [u8]) {
        self.out_index = 0;
        //println!("in_samples.len() = {}", in_samples.len());
        //println!("out_samples.len() = {}", out_samples.len());

        for n in 0..in_samples.len() {
            let sn = in_samples[n];
            self.history.push_back(sn);
    
            if self.history.len() >= self.samples_per_symbol {
                let s_n_l = self.history.front().copied().unwrap();
                let l = self.samples_per_symbol as f32;
                let r_l = self.r.powf(l);
    
                let omega0_t = self.omega_mark * self.sampling_period;
                let omega1_t = self.omega_space * self.sampling_period;
                let omega0_lt = self.omega_mark * l * self.sampling_period;
                let omega1_lt = self.omega_space * l * self.sampling_period;
                
                let v0r = sn - r_l * (omega0_lt.cos()) * s_n_l + self.r * (omega0_t.cos()) * self.v0r_last - self.r * (omega0_t.sin()) * self.v0i_last;
    
                let v0i = -r_l * (omega0_lt.sin()) * s_n_l + self.r * (omega0_t.cos()) * self.v0i_last + self.r * (omega0_t.sin()) * self.v0r_last;
    
                let v1r = sn
                    - r_l * (omega1_lt.cos()) * s_n_l
                    + self.r * (omega1_t.cos()) * self.v1r_last
                    - self.r * (omega1_t.sin()) * self.v1i_last;
    
                let v1i = -r_l * (omega1_lt.sin()) * s_n_l
                    + self.r * (omega1_t.cos()) * self.v1i_last
                    + self.r * (omega1_t.sin()) * self.v1r_last;
    
                // Atualiza os estados
                self.v0r_last = v0r;
                self.v0i_last = v0i;
                self.v1r_last = v1r;
                self.v1i_last = v1i;
    
                // Decide bit (comparação de energia)
                let energy0 = v0r * v0r + v0i * v0i;
                let energy1 = v1r * v1r + v1i * v1i;
                let total_energy = energy0 + energy1;
                let threshold = 1e-7;

                if total_energy < threshold {
                    // Sem portadora
                    out_samples[self.out_index] = 0; // Ou outro valor para indicar erro
                } else {
                    out_samples[self.out_index] = if energy1 > energy0 { 1 } else { 0 };
                }

                self.out_index += 1;
    
                self.history.pop_front(); // remove s[n - L]
            }
        }
    }
}

pub struct V21TX {
    sampling_period: f32,
    omega_mark: f32,
    omega_space: f32,
    phase: f32,
}

impl V21TX {
    pub fn new(sampling_period: f32, omega_mark: f32, omega_space: f32) -> Self {
        Self {
            sampling_period,
            omega_mark,
            omega_space,
            phase: 0.,
        }
    }

    pub fn modulate(&mut self, in_samples: &[u8], out_samples: &mut [f32]) {
        debug_assert!(in_samples.len() == out_samples.len());

        for i in 0..in_samples.len() {
            out_samples[i] = self.phase.sin();

            let omega = if in_samples[i] == 0 {
                self.omega_space
            } else {
                self.omega_mark
            };
            self.phase = (self.phase + self.sampling_period * omega).rem(2. * PI);
        }
    }
}
