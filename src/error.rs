use embedded_hal::i2c;

#[derive(Debug)]
pub enum ModelError {
    InvalidValueU8(u8),
}

#[derive(Debug)]
pub enum OperationError<Error: i2c::Error> {
    I2CError(Error),
    ModelError(ModelError),
}
