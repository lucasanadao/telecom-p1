use crossbeam_channel::Sender;
use std::collections::VecDeque;

pub struct UartRx {
    samples_per_symbol: usize,
    to_pty: Sender<u8>,
    history: VecDeque<u8>,
    state: RxState,
    sample_count: usize,
    bit_index: usize,
    current_byte: u8,
    mid_bit_counter: usize,
}

enum RxState {
    Idle,
    Receiving,
    StopBit,
    MidBit,
}

impl UartRx {
    pub fn new(samples_per_symbol: usize, to_pty: Sender<u8>) -> Self {
        Self {
            samples_per_symbol,
            to_pty,
            history: VecDeque::with_capacity(30),
            state: RxState::Idle,
            mid_bit_counter: 0,
            sample_count: 0,
            bit_index: 0,
            current_byte: 0,
        }
    }

    pub fn put_samples(&mut self, buffer: &[u8]) {
        for &sample in buffer {
            self.history.push_back(sample);
            if self.history.len() > 30 {
                self.history.pop_front();
            }

            match self.state {
                RxState::Idle => {
                    if sample == 0 && self.history.len() == 30 {
                        let low_count = self.history.iter().filter(|&&s| s == 0).count();
                        if low_count >= 25 && *self.history.front().unwrap() == 0 {
                            self.bit_index = 0;
                            self.sample_count = 0;
                            self.mid_bit_counter = 0;
                            self.current_byte = 0;
                            self.state = RxState::MidBit;
                        }
                    }
                }

                RxState::MidBit => {
                    self.mid_bit_counter += 1;
                    if self.mid_bit_counter >= 50 {
                        self.state = RxState::Receiving;
                    }
                }

                RxState::Receiving => {
                    self.sample_count += 1;
                    if self.sample_count == (self.bit_index + 1) * self.samples_per_symbol {
                        // Pega valor no meio do símbolo
                        self.current_byte |= sample << self.bit_index;
                        self.bit_index += 1;

                        if self.bit_index >= 8 {
                            self.state = RxState::StopBit;
                        }
                    }
                }

                RxState::StopBit => {
                    self.sample_count += 1;
                    if self.sample_count == 9 * self.samples_per_symbol {
                        let _ = self.to_pty.send(self.current_byte);
                        self.state = RxState::Idle;
                        self.history.clear();
                    }
                }
            }
        }
    }
}

pub struct UartTx {
    samples_per_symbol: usize,
    samples: VecDeque<u8>,
}

impl UartTx {
    pub fn new(samples_per_symbol: usize) -> Self {
        Self {
            samples_per_symbol,
            samples: VecDeque::new(),
        }
    }

    fn put_bit(&mut self, bit: u8) {
        for _ in 0..self.samples_per_symbol {
            self.samples.push_back(bit);
        }
    }

    pub fn put_byte(&mut self, mut byte: u8) {
        self.put_bit(0); // start bit
        for _ in 0..8 {
            self.put_bit(byte & 1);
            byte >>= 1;
        }
        self.put_bit(1); // stop bit
    }

    pub fn get_samples(&mut self, buffer: &mut [u8]) {
        for i in 0..buffer.len() {
            buffer[i] = self.samples.pop_front().unwrap_or(1);
        }
    }
}
