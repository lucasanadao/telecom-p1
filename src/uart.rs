use crossbeam_channel::Sender;
use std::collections::VecDeque;

pub struct UartRx {
    // TODO: coloque outros atributos que você precisar aqui
    samples_per_symbol: usize,
    to_pty: Sender<u8>,
    byte: u8,
}

impl UartRx {
    pub fn new(samples_per_symbol: usize, to_pty: Sender<u8>) -> Self {
        // TODO: inicialize seus novos atributos abaixo
        UartRx {
            samples_per_symbol,
            to_pty,
            byte: 0,
        }
    }

    pub fn put_samples(&mut self, buffer: &[u8]) {
        let mut pointer = 0;
    
        while pointer + 160 * 10 < buffer.len() {
            let mut count = 0;

            for i in 0..30 {
                if buffer[pointer + i] == 0 {
                    count += 1;
                }
            }
    
            if count >= 25 {
                pointer += 80;
                self.byte = 0;

                for bit in 0..8 {
                    let sample = buffer[pointer + (bit + 1) * 160];
                    if sample & 1 != 0 {
                        self.byte |= 1 << (7 - bit);
                    }
                }

                self.to_pty.send(self.byte).unwrap();
    
                pointer += 160 * 9;
            } else {
                pointer += 1;
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
