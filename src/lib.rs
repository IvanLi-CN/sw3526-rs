#![cfg_attr(not(test), no_std)]

use embedded_hal::i2c;
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

mod model;
pub use model::*;
mod error;
pub use error::*;

static ADDRESS: u8 = 0x3c; // fixed address

pub struct SW3526<I2C> {
    i2c: I2C,
    adc_config: Option<AdcConfig>,
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "SW3526",),
    async(feature = "async", keep_self)
)]
impl<I2C, E> SW3526<I2C>
where
    E: i2c::Error,
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            adc_config: None,
        }
    }

    pub async fn get_chip_version(&mut self) -> Result<u8, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::VersionInfo as u8], &mut buf)
            .await?;
        Ok(buf[0] & 0x03)
    }

    pub async fn get_buck_output_millivolts(&mut self) -> Result<u16, E> {
        let mut buf_h = [0u8; 1];
        let mut buf_l = [0u8; 1];

        self.i2c
            .write_read(
                ADDRESS,
                &[Register::BuckOutputVoltageHigh8b as u8],
                &mut buf_h,
            )
            .await?;
        self.i2c
            .write_read(
                ADDRESS,
                &[Register::BuckOutputVoltageLow4b as u8],
                &mut buf_l,
            )
            .await?;

        Ok((((buf_h[0] as u16) << 4) | ((buf_l[0] as u16) >> 4)) * 10)
    }

    pub async fn get_buck_output_limit_milliamps(&mut self) -> Result<u16, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::BuckOutputCurrentLimit as u8], &mut buf)
            .await?;

        Ok(1000u16 + ((buf[0] as u16) & 0x3f) * 50)
    }

    pub async fn get_protocol(&mut self) -> Result<ProtocolIndicationResponse, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::ProtocolIndication as u8], &mut buf)
            .await?;

        Ok(buf[0].into())
    }

    pub async fn get_system_status(&mut self) -> Result<SystemStatusResponse, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::SystemStatus as u8], &mut buf)
            .await?;

        Ok(buf[0].into())
    }

    pub async fn get_abnormal_case(&mut self) -> Result<AbnormalCaseResponse, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AbnormalCase as u8], &mut buf)
            .await?;

        Ok(buf[0].into())
    }

    /// set i2c writable
    /// After doing following step then reg0xA0~BF, reg0x70~71 and reg0x13 can be written by MCU
    pub async fn set_i2c_writable(&mut self) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::I2cEnable as u8, 0x20, 0x40, 0x80])
            .await
    }

    pub async fn get_buck_force_off(&mut self) -> Result<BuckForceOffConfig, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::BuckForceOff as u8], &mut buf)
            .await?;

        Ok(buf[0].into())
    }

    pub async fn set_buck_force_off(&mut self, config: BuckForceOffConfig) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::BuckForceOff as u8, config.into()])
            .await
    }

    pub async fn get_adc_input_millivolts(&mut self) -> Result<u16, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AdcVinData as u8], &mut buf)
            .await?;
        Ok(((buf[0] as u16) << 4) * 10)
    }

    pub async fn get_adc_output_millivolts(&mut self) -> Result<u16, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AdcVoutData as u8], &mut buf)
            .await?;
        Ok(((buf[0] as u16) << 4) * 6)
    }

    pub async fn get_adc_output_milliamps(&mut self) -> Result<f32, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AdcIoutData as u8], &mut buf)
            .await?;
        Ok((((buf[0] as u16) << 4) as f32) * 2.5)
    }

    pub async fn get_adc_config(&mut self) -> Result<AdcConfig, E> {
        let mut buf = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AdcConfig as u8], &mut buf)
            .await?;

        let config = AdcConfig::try_from(buf[0] & 0x03).unwrap();

        self.adc_config = Some(config);

        Ok(config)
    }

    /// Set the ADC data type
    pub async fn set_adc_config(&mut self, config: AdcConfig) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::AdcConfig as u8, config as u8])
            .await?;

        self.adc_config = Some(config);

        Ok(())
    }

    /// Get the ADC value in raw format
    /// The meaning of the value representation is specified by reg 0x31
    pub async fn get_adc_data_raw(&mut self) -> Result<u16, E> {
        let mut buf_h = [0u8; 1];
        let mut buf_l = [0u8; 1];

        self.i2c
            .write_read(ADDRESS, &[Register::AdcDataHigh8b as u8], &mut buf_h)
            .await?;
        self.i2c
            .write_read(ADDRESS, &[Register::AdcDataLow4b as u8], &mut buf_l)
            .await?;

        Ok(((buf_h[0] as u16) << 4) | ((buf_l[0] & 0x0f) as u16))
    }

    /// Get the ADC value in millivolts or milliamps
    /// The meaning of the value representation is specified by reg 0x31
    /// Returns None if the ADC config is not set through `set_adc_config`
    pub async fn get_adc_data(&mut self) -> Result<Option<f32>, E> {
        if self.adc_config.is_none() {
            return Ok(None);
        }

        let raw = self.get_adc_data_raw().await?;

        let value = match self.adc_config.unwrap() {
            AdcConfig::Vin => (raw as f32) * 10.0,
            AdcConfig::Vout => (raw as f32) * 6.0,
            AdcConfig::Iout => (raw as f32) * 2.5,
        };

        Ok(Some(value))
    }

    /// Get this register indicates the power value set by Rset or reg0xA7.
    /// Value range is 0x00 to 0x7F, units are watts
    pub async fn get_limit_watts(&mut self) -> Result<u8, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::PowerStatus as u8], &mut buf)
            .await?;
        Ok(buf[0] & 0x7f)
    }

    /// get CC status
    pub async fn get_cc_status(&mut self) -> Result<CcStatus, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::CcStatus as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// get power command request
    pub async fn get_power_command_request(
        &mut self,
    ) -> Result<PowerCommandRequest, OperationError<E>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::PowerCommandRequest as u8], &mut buf)
            .await
            .map_err(|e| OperationError::I2CError(e))?;
        buf[0].try_into().map_err(|e| OperationError::ModelError(e))
    }

    /// set power command request
    pub async fn set_power_command_request(
        &mut self,
        config: PowerCommandRequest,
    ) -> Result<(), E> {
        self.i2c
            .write(
                ADDRESS,
                &[
                    Register::PowerCommandRequest as u8,
                    ((config.send_enabled as u8) << 7) | config.command as u8,
                ],
            )
            .await
    }

    /// send pd hard reset
    pub async fn send_pd_hard_reset(&mut self) -> Result<(), E> {
        self.set_power_command_request(PowerCommandRequest {
            send_enabled: true,
            command: PdCommand::HardReset,
        })
        .await
    }

    /// get fast charge config 6
    pub async fn get_fast_charge_config_6(&mut self) -> Result<FastChargeConfig6, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig6 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 6
    pub async fn set_fast_charge_config_6(&mut self, config: FastChargeConfig6) -> Result<(), E> {
        self.i2c
            .write(
                ADDRESS,
                &[
                    Register::FastChargeConfig6 as u8,
                    config.into(),
                ],
            )
            .await
    }

    /// get fast charge config 5
    pub async fn get_fast_charge_config_5(&mut self) -> Result<FastChargeConfig5, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig5 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 5
    pub async fn set_fast_charge_config_5(&mut self, config: FastChargeConfig5) -> Result<(), E> {
        self.i2c
            .write(
                ADDRESS,
                &[
                    Register::FastChargeConfig5 as u8,
                    config.into(),
                ],
            )
            .await
    }

    /// get power config
    pub async fn get_output_limit_watts(&mut self) -> Result<u8, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::PowerConfig as u8], &mut buf)
            .await?;

        let raw = buf[0] & 0x3f;
        if raw < 8 {
            Ok(raw + 64)
        } else {
            Ok(raw)
        }
    }

    /// set power config
    /// Value range is [12, 71].
    pub async fn set_output_limit_watts(&mut self, watts: u8) -> Result<(), E> {
        let raw = if watts < 8 { watts - 64 } else { watts };

        self.i2c
            .write(ADDRESS, &[Register::PowerConfig as u8, raw])
            .await
    }

    /// get fast charge config 0
    pub async fn get_fast_charge_config_0(&mut self) -> Result<FastChargeConfig0, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig0 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 0
    pub async fn set_fast_charge_config_0(&mut self, config: FastChargeConfig0) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::FastChargeConfig0 as u8, config.into()])
            .await
    }

    /// get fast charge config 1
    pub async fn get_fast_charge_config_1(&mut self) -> Result<FastChargeConfig1, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig1 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 1
    pub async fn set_fast_charge_config_1(&mut self, config: FastChargeConfig1) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::FastChargeConfig1 as u8, config.into()])
            .await
    }

    /// get fast charge config 2
    pub async fn get_fast_charge_config_2(&mut self) -> Result<FastChargeConfig2, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig2 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 2
    pub async fn set_fast_charge_config_2(&mut self, config: FastChargeConfig2) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::FastChargeConfig2 as u8, config.into()])
            .await
    }

    /// get fast charge config 3
    pub async fn get_fast_charge_config_3(&mut self) -> Result<FastChargeConfig3, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig3 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 3
    pub async fn set_fast_charge_config_3(&mut self, config: FastChargeConfig3) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::FastChargeConfig3 as u8, config.into()])
            .await
    }

    /// get fast charge config 4
    pub async fn get_fast_charge_config_4(&mut self) -> Result<FastChargeConfig4, E> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::FastChargeConfig4 as u8], &mut buf)
            .await?;
        Ok(buf[0].into())
    }

    /// set fast charge config 4
    pub async fn set_fast_charge_config_4(&mut self, config: FastChargeConfig4) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::FastChargeConfig4 as u8, config.into()])
            .await
    }

    /// get USB VID
    pub async fn get_vid(&mut self) -> Result<u16, E> {
        let mut buf_l = [0u8; 1];
        let mut buf_h = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[Register::VidConfig0 as u8], &mut buf_l)
            .await?;
        self.i2c
            .write_read(ADDRESS, &[Register::VidConfig1 as u8], &mut buf_h)
            .await?;
        Ok(((buf_h[0] as u16) << 8) | buf_l[0] as u16)
    }

    /// set USB VID
    pub async fn set_vid(&mut self, vid: u16) -> Result<(), E> {
        self.i2c
            .write(ADDRESS, &[Register::VidConfig0 as u8, (vid >> 8) as u8])
            .await?;
        self.i2c
            .write(ADDRESS, &[Register::VidConfig1 as u8, vid as u8])
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

    #[test]
    fn get_chip_version() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x01], vec![0x01])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let version = sw3526.get_chip_version().unwrap();

        assert!(version == 0x01);

        i2c.done();
    }

    #[test]
    fn get_buck_output_millivolts() {
        let i2c_expectations = [
            Transaction::write_read(ADDRESS, vec![0x03], vec![0xff]), // eq 0xff
            Transaction::write_read(ADDRESS, vec![0x04], vec![0xff]),
        ]; // eq 0xf0
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let output = sw3526.get_buck_output_millivolts().unwrap();

        assert!(output == 40950);

        i2c.done();
    }

    #[test]
    fn get_buck_output_limit_milliamps() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x05], vec![0xff])]; // eq 0x3f
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let limit = sw3526.get_buck_output_limit_milliamps().unwrap();

        assert!(limit == 4150);

        i2c.done();
    }

    #[test]
    fn get_protocol_1() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x06], vec![0xaa])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let res = sw3526.get_protocol().unwrap();

        assert!(matches!(res.protocol_status, ProtocolStatus::OnLine));
        assert!(matches!(res.voltage_status, VoltageStatus::_5V));
        assert!(matches!(res.pd_version, PdVersion::PD3_0));
        assert!(matches!(res.protocol, ProtocolIndication::SFCP));

        i2c.done();
    }

    #[test]
    fn get_protocol_2() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x06], vec![0x55])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let res = sw3526.get_protocol().unwrap();

        assert!(matches!(res.protocol_status, ProtocolStatus::OffLine));
        assert!(matches!(res.voltage_status, VoltageStatus::ProtocolVoltage));
        assert!(matches!(res.pd_version, PdVersion::PD2_0));
        assert!(matches!(res.protocol, ProtocolIndication::PdFix));

        i2c.done();
    }

    #[test]
    fn get_abnormal_case_normal() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x0b], vec![0xe8])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let abnormal_case = sw3526.get_abnormal_case().unwrap();

        assert!(matches!(abnormal_case.vin_ovp_status, VinOvpStatus::Normal));
        assert!(matches!(
            abnormal_case.over_temperature_alarm_status,
            OverTemperatureAlarmStatus::Normal
        ));
        assert!(matches!(
            abnormal_case.over_temperature_shutdown_status,
            OverTemperatureShutdownStatus::Normal
        ));
        assert!(matches!(
            abnormal_case.output_short_circuit_status,
            OutputShortCircuitStatus::Normal
        ));

        i2c.done();
    }

    #[test]
    fn get_abnormal_case_abnormal() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x0b], vec![0x17])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let abnormal_case = sw3526.get_abnormal_case().unwrap();

        assert!(matches!(abnormal_case.vin_ovp_status, VinOvpStatus::Ovp));
        assert!(matches!(
            abnormal_case.over_temperature_alarm_status,
            OverTemperatureAlarmStatus::Alarm
        ));
        assert!(matches!(
            abnormal_case.over_temperature_shutdown_status,
            OverTemperatureShutdownStatus::Shutdown
        ));
        assert!(matches!(
            abnormal_case.output_short_circuit_status,
            OutputShortCircuitStatus::Short
        ));

        i2c.done();
    }

    #[test]
    fn set_i2c_writable() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0x12, 0x20, 0x40, 0x80])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526.set_i2c_writable().unwrap();

        i2c.done();
    }

    #[test]
    fn get_buck_force_off() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x13], vec![0x80])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let buck_force_off = sw3526.get_buck_force_off().unwrap();

        assert!(matches!(
            buck_force_off.force_off,
            BuckForceOff::TurnOffOneSecond
        ));
        assert!(matches!(
            buck_force_off.cc_un_driven_duration_buck_force_off,
            CCUnDrivenDurationBuckForceOff::Driven
        ));

        i2c.done();
    }

    #[test]
    fn set_buck_force_off() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0x13, 0xc0])];
        let mut i2c = Mock::new(&i2c_expectations);

        let mut sw3526 = SW3526::new(i2c.clone());
        sw3526
            .set_buck_force_off(BuckForceOffConfig {
                force_off: BuckForceOff::TurnOffOneSecond,
                cc_un_driven_duration_buck_force_off: CCUnDrivenDurationBuckForceOff::UnDriven,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_adc_input_millivolts() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x30], vec![0xff])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let input = sw3526.get_adc_input_millivolts().unwrap();

        assert!(input == 40800);

        i2c.done();
    }

    #[test]
    fn get_adc_output_millivolts() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x31], vec![0xff])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let output = sw3526.get_adc_output_millivolts().unwrap();

        assert!(output == 24480);

        i2c.done();
    }

    #[test]
    fn get_adc_output_milliamps() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x33], vec![0xff])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let output = sw3526.get_adc_output_milliamps().unwrap();

        assert!(output == 10200f32);

        i2c.done();
    }

    #[test]
    fn get_adc_config() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x3a], vec![0xff])]; // eq 0x3
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let config = sw3526.get_adc_config().unwrap();

        assert!(matches!(config, AdcConfig::Iout));

        i2c.done();
    }

    #[test]
    fn set_adc_config() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0x3a, 0x01])];
        let mut i2c = Mock::new(&i2c_expectations);

        let mut sw3526 = SW3526::new(i2c.clone());
        sw3526.set_adc_config(AdcConfig::Vin).unwrap();

        i2c.done();
    }

    #[test]
    fn get_adc_data_raw() {
        let i2c_expectations = [
            Transaction::write_read(ADDRESS, vec![0x3b], vec![0xff]), // eq 0xff
            Transaction::write_read(ADDRESS, vec![0x3c], vec![0xff]), // eq 0x0f
        ];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let data = sw3526.get_adc_data_raw().unwrap();

        assert!(data == 4095);

        i2c.done();
    }

    #[test]
    fn get_adc_data_when_adc_config_not_configured() {
        let i2c_expectations = [];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let data = sw3526.get_adc_data().unwrap();

        assert!(data.is_none());

        i2c.done();
    }

    #[test]
    fn get_adc_data_when_adc_config_configured() {
        let i2c_expectations = [
            Transaction::write(ADDRESS, vec![0x3a, 0x01]),
            Transaction::write_read(ADDRESS, vec![0x3b], vec![0xff]), // eq 0xff
            Transaction::write_read(ADDRESS, vec![0x3c], vec![0xff]), // eq 0x0f
        ];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526.set_adc_config(AdcConfig::Vin).unwrap();
        let data = sw3526.get_adc_data().unwrap();

        assert!(data.is_some());
        assert!(data.unwrap() == 40950_f32);

        i2c.done();
    }

    #[test]
    fn get_cc_status() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x69], vec![0xff])]; // eq 0x30
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let status = sw3526.get_cc_status().unwrap();

        assert!(status.cc1_attached == true);
        assert!(status.cc2_attached == true);

        i2c.done();
    }

    #[test]
    fn get_power_command_request() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0x70], vec![0xf9])]; // eq 0x81
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let status = sw3526.get_power_command_request().unwrap();

        assert!(status.send_enabled == true);
        assert!(matches!(status.command, PdCommand::HardReset));

        i2c.done();
    }

    #[test]
    fn set_power_command_request() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0x70, 0x81])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_power_command_request(PowerCommandRequest {
                send_enabled: true,
                command: PdCommand::HardReset,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn send_pd_hard_reset() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0x70, 0x81])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526.send_pd_hard_reset().unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_6() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xa2], vec![0xff])]; // eq 0x60
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_6().unwrap();

        assert!(fast_charge_config.qc2_0_qc3_0_cable_compatible_and_offset_enabled == true);
        assert!(fast_charge_config.pdo_link_with_vin == true);

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_6() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xa2, 0x60])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_6(FastChargeConfig6 {
                qc2_0_qc3_0_cable_compatible_and_offset_enabled: true,
                pdo_link_with_vin: true,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_5() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xa4], vec![0xff])]; // eq 0x60
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_5().unwrap();

        assert!(matches!(
            fast_charge_config.scp_select,
            ScpSelect::HighVoltage
        ));
        assert!(matches!(
            fast_charge_config.pe2_0_max_voltage,
            Pe2_0MaxVoltage::_20V
        ));

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_5() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xa4, 0x60])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_5(FastChargeConfig5 {
                scp_select: ScpSelect::HighVoltage,
                pe2_0_max_voltage: Pe2_0MaxVoltage::_20V,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_output_limit_watts() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xa7], vec![0xff])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let output = sw3526.get_output_limit_watts().unwrap();

        assert!(output == 63);

        i2c.done();
    }

    #[test]
    fn set_output_limit_watts() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xa7, 0x3f])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526.set_output_limit_watts(63).unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_0() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xa8], vec![0xaa])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_0().unwrap();

        assert!(fast_charge_config.scp_enabled == true);
        assert!(fast_charge_config.vooc_enabled == false);
        assert!(fast_charge_config.sfcp_enabled == true);
        assert!(fast_charge_config.qc2_0_enabled == false);
        assert!(fast_charge_config.qc3_0_enabled == true);
        assert!(fast_charge_config.fcp_enabled == false);
        assert!(fast_charge_config.afc_enabled == true);
        assert!(fast_charge_config.pe_enabled == false);

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_0() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xa8, 0x55])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_0(FastChargeConfig0 {
                scp_enabled: false,
                vooc_enabled: true,
                sfcp_enabled: false,
                qc2_0_enabled: true,
                qc3_0_enabled: false,
                fcp_enabled: true,
                afc_enabled: false,
                pe_enabled: true,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_1() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xa9], vec![0xaa])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_1().unwrap();

        assert!(fast_charge_config.pps1_enabled == true);
        assert!(fast_charge_config.pps0_enabled == false);
        assert!(fast_charge_config.pd_20v_enabled == true);
        assert!(fast_charge_config.pd_15v_enabled == false);
        assert!(fast_charge_config.pd_12v_enabled == true);
        assert!(fast_charge_config.pd_9v_enabled == false);
        assert!(fast_charge_config.pd_enabled == false);

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_1() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xa9, 0x55])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_1(FastChargeConfig1 {
                pps1_enabled: false,
                pps0_enabled: true,
                pd_20v_enabled: false,
                pd_15v_enabled: true,
                pd_12v_enabled: false,
                pd_9v_enabled: true,
                pd_enabled: true,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_2() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xaa], vec![0xff])]; // eq 0x23
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_2().unwrap();

        assert!(fast_charge_config.dpdm_enabled == true);
        assert!(matches!(
            fast_charge_config.max_output_voltage_except_pd,
            MaxOutputVoltageExceptPd::_20V
        ));

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_2() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xaa, 0x23])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_2(FastChargeConfig2 {
                dpdm_enabled: true,
                max_output_voltage_except_pd: MaxOutputVoltageExceptPd::_20V,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_3() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xab], vec![0xaa])]; // eq 0xa4
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_3().unwrap();

        // pd_current_limit_protect_method: (value & 0x80).try_into().unwrap(),
        //     qc3_0_current_limit_protect_method: (value & 0x40).try_into().unwrap(),
        //     qc3_0_constant_power_enabled: (value & 0x20) != 0,
        //     pps_constant_power_enabled: (value & 0x10) != 0,
        //     power_setting_method: (value & 0x01).try_into().unwrap(),
        assert!(matches!(
            fast_charge_config.pd_current_limit_protect_method,
            PdCurrentLimitProtectMethod::OC
        ));
        assert!(matches!(
            fast_charge_config.qc3_0_current_limit_protect_method,
            QC3_0CurrentLimitProtectMethod::CCLoop
        ));
        assert!(fast_charge_config.qc3_0_constant_power_enabled == true);
        assert!(fast_charge_config.pps_constant_power_enabled == false);
        assert!(matches!(
            fast_charge_config.power_setting_method,
            PowerSettingMethod::Rset
        ));

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_3() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xab, 0x54])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_3(FastChargeConfig3 {
                pd_current_limit_protect_method: PdCurrentLimitProtectMethod::UV,
                qc3_0_current_limit_protect_method: QC3_0CurrentLimitProtectMethod::VoltageDrop,
                qc3_0_constant_power_enabled: false,
                pps_constant_power_enabled: true,
                power_setting_method: PowerSettingMethod::Register,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_fast_charge_config_4() {
        let i2c_expectations = [Transaction::write_read(ADDRESS, vec![0xac], vec![0x05])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let fast_charge_config = sw3526.get_fast_charge_config_4().unwrap();

        assert!(fast_charge_config.port_fast_charge_disabled == true);
        assert!(fast_charge_config.pd_5v_2a_rebroadcast_enabled == true);

        i2c.done();
    }

    #[test]
    fn set_fast_charge_config_4() {
        let i2c_expectations = [Transaction::write(ADDRESS, vec![0xac, 0x00])];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526
            .set_fast_charge_config_4(FastChargeConfig4 {
                port_fast_charge_disabled: false,
                pd_5v_2a_rebroadcast_enabled: false,
            })
            .unwrap();

        i2c.done();
    }

    #[test]
    fn get_vid() {
        let i2c_expectations = [
            Transaction::write_read(ADDRESS, vec![0xae], vec![0xaa]),
            Transaction::write_read(ADDRESS, vec![0xaf], vec![0xaa]),
        ];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        let vid = sw3526.get_vid().unwrap();

        assert!(vid == 0xaaaa);

        i2c.done();
    }

    #[test]
    fn set_vid() {
        let i2c_expectations = [
            Transaction::write(ADDRESS, vec![0xae, 0xaa]),
            Transaction::write(ADDRESS, vec![0xaf, 0xaa]),
        ];
        let mut i2c = Mock::new(&i2c_expectations);
        let mut sw3526 = SW3526::new(i2c.clone());

        sw3526.set_vid(0xaaaa).unwrap();

        i2c.done();
    }
}
