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
    
        while pointer + 160 * 9 < buffer.len() {
            // Possível início de start bit
            if buffer[pointer] == 0 {
                // Verifica se pelo menos 25 das 30 amostras próximas ao meio são 0
                let mut count = 0;
                for j in 0..30 {
                    if buffer[pointer + 65 + j] == 0 {
                        count += 1;
                    }
                }
    
                if count >= 25 {
                    // Meio do start bit
                    let mut byte = 0;
                    count = 0;

                    while buffer[pointer] != 1{
                        for j in 0..30 {
                            if buffer[pointer - j]  == 1{
                                count += 1;
                            }
                        }

                        if count >= 25 {
                            break;
                        }

                        pointer -= 1;
                    }

                    let mid = pointer + self.samples_per_symbol/2;
    
                    for bit_index in 0..8 {
                        let sample = buffer[mid + (bit_index + 1) * self.samples_per_symbol] & 1;
                        byte |= sample << bit_index;
                    }
    
                    self.to_pty.send(byte).unwrap();
                    pointer = mid + 9 * self.samples_per_symbol; // pula todo o símbolo
                    continue;
                }
            }
    
            pointer += 1;
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
