//! Lock-free ring buffer for audio data
//! 
//! Thread-safe circular buffer for passing audio between threads

use ringbuf::{HeapRb, HeapConsumer, HeapProducer};

/// Thread-safe audio ring buffer
pub struct AudioRingBuffer {
    producer: HeapProducer<f64>,
    consumer: HeapConsumer<f64>,
    capacity: usize,
}

impl AudioRingBuffer {
    /// Create new ring buffer with given capacity
    /// 
    /// # Arguments
    /// * `capacity` - Buffer capacity in samples
    pub fn new(capacity: usize) -> Self {
        let rb = HeapRb::<f64>::new(capacity);
        let (producer, consumer) = rb.split();
        
        Self {
            producer,
            consumer,
            capacity,
        }
    }
    
    /// Split into producer and consumer ends
    pub fn split(self) -> (AudioProducer, AudioConsumer) {
        (
            AudioProducer {
                producer: self.producer,
                capacity: self.capacity,
            },
            AudioConsumer {
                consumer: self.consumer,
                capacity: self.capacity,
            },
        )
    }
    
    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Producer end of audio ring buffer (for writing)
pub struct AudioProducer {
    producer: HeapProducer<f64>,
    capacity: usize,
}

impl AudioProducer {
    /// Write samples to buffer
    /// 
    /// # Arguments
    /// * `samples` - Samples to write
    /// 
    /// # Returns
    /// Number of samples actually written (may be less if buffer is full)
    pub fn write(&mut self, samples: &[f64]) -> usize {
        self.producer.push_slice(samples)
    }
    
    /// Check if buffer has space for n samples
    pub fn has_space(&self, n: usize) -> bool {
        self.producer.free_len() >= n
    }
    
    /// Get number of free slots
    pub fn free_len(&self) -> usize {
        self.producer.free_len()
    }
    
    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Consumer end of audio ring buffer (for reading)
pub struct AudioConsumer {
    consumer: HeapConsumer<f64>,
    capacity: usize,
}

impl AudioConsumer {
    /// Read samples from buffer
    /// 
    /// # Arguments
    /// * `buffer` - Output buffer to read into
    /// 
    /// # Returns
    /// Number of samples actually read (may be less if buffer doesn't have enough)
    pub fn read(&mut self, buffer: &mut [f64]) -> usize {
        self.consumer.pop_slice(buffer)
    }
    
    /// Read exactly n samples, blocking until available
    /// 
    /// NOTE: This will spin-wait, use only in real-time audio threads
    pub fn read_exact(&mut self, buffer: &mut [f64]) -> usize {
        let mut total_read = 0;
        while total_read < buffer.len() {
            let n = self.consumer.pop_slice(&mut buffer[total_read..]);
            total_read += n;
            if n == 0 {
                std::thread::yield_now();
            }
        }
        total_read
    }
    
    /// Check if buffer has n samples available
    pub fn has_data(&self, n: usize) -> bool {
        self.consumer.len() >= n
    }
    
    /// Get number of available samples
    pub fn len(&self) -> usize {
        self.consumer.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }
    
    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer_write_read() {
        let rb = AudioRingBuffer::new(1024);
        let (mut producer, mut consumer) = rb.split();
        
        // Write some data
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let written = producer.write(&data);
        assert_eq!(written, 5);
        
        // Read it back
        let mut output = vec![0.0; 5];
        let read = consumer.read(&mut output);
        assert_eq!(read, 5);
        assert_eq!(output, data);
    }
    
    #[test]
    fn test_ring_buffer_overflow() {
        let rb = AudioRingBuffer::new(10);
        let (mut producer, mut consumer) = rb.split();
        
        // Try to write more than capacity
        let data = vec![1.0; 20];
        let written = producer.write(&data);
        
        // Should only write up to capacity
        assert!(written <= 10);
        
        // Read back
        let mut output = vec![0.0; 20];
        let read = consumer.read(&mut output);
        assert_eq!(read, written);
    }
    
    #[test]
    fn test_ring_buffer_underflow() {
        let rb = AudioRingBuffer::new(1024);
        let (mut _producer, mut consumer) = rb.split();
        
        // Try to read from empty buffer
        let mut output = vec![0.0; 10];
        let read = consumer.read(&mut output);
        
        // Should read 0
        assert_eq!(read, 0);
    }
}
