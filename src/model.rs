use num_enum::TryFromPrimitive;

type ModelError = crate::error::ModelError;

#[repr(u8)]
pub enum Register {
    VersionInfo = 0x01,
    BuckOutputVoltageHigh8b = 0x03,
    BuckOutputVoltageLow4b = 0x04,
    BuckOutputCurrentLimit = 0x05,
    ProtocolIndication = 0x06,
    SystemStatus = 0x07,
    AbnormalCase = 0x0B,
    I2cEnable = 0x12,
    BuckForceOff = 0x13,
    AdcVinData = 0x30,
    AdcVoutData = 0x31,
    AdcIoutData = 0x33,
    AdcConfig = 0x3A,
    AdcDataHigh8b = 0x3B,
    AdcDataLow4b = 0x3C,
    PowerStatus = 0x68,
    CcStatus = 0x69,
    PowerCommandRequest = 0x70,
    FastChargeConfig6 = 0xA2,
    FastChargeConfig5 = 0xA4,
    PowerConfig = 0xA7,
    FastChargeConfig0 = 0xA8,
    FastChargeConfig1 = 0xA9,
    FastChargeConfig2 = 0xAA,
    FastChargeConfig3 = 0xAB,
    FastChargeConfig4 = 0xAC,
    VidConfig0 = 0xAE,
    VidConfig1 = 0xAF,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum ProtocolStatus {
    OffLine = 0,
    OnLine = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum VoltageStatus {
    _5V = 0,
    ProtocolVoltage = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PdVersion {
    Unknown = 0,
    PD2_0 = 1,
    PD3_0 = 2,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum ProtocolIndication {
    Unknown = 0,
    QC2_0 = 1,
    QC3_0 = 2,
    FCP = 3,
    SCP = 4,
    PdFix = 5,
    PdPps = 6,
    PE1_1 = 7,
    PE2_0 = 8,
    VOOC = 9,
    SFCP = 10,
    AFC = 11,
}

#[derive(Debug)]
pub struct ProtocolIndicationResponse {
    pub protocol_status: ProtocolStatus,
    pub voltage_status: VoltageStatus,
    pub pd_version: PdVersion,
    pub protocol: ProtocolIndication,
}

impl From<u8> for ProtocolIndicationResponse {
    fn from(value: u8) -> Self {
        Self {
            protocol_status: ((value & 0x80) >> 7).try_into().unwrap(),
            voltage_status: ((value & 0x40) >> 6).try_into().unwrap(),
            pd_version: ((value & 0x30) >> 4)
                .try_into()
                .unwrap_or(PdVersion::Unknown),
            protocol: (value & 0x0F)
                .try_into()
                .unwrap_or(ProtocolIndication::Unknown),
        }
    }
}

impl From<ProtocolIndicationResponse> for u8 {
    fn from(value: ProtocolIndicationResponse) -> Self {
        (value.protocol_status as u8) << 7
            | (value.voltage_status as u8) << 6
            | (value.pd_version as u8) << 4
            | (value.protocol as u8)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PortStatus {
    Off = 0,
    On = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum BuckStatus {
    Off = 0,
    On = 1,
}

#[derive(Debug)]
pub struct SystemStatusResponse {
    pub port_status: PortStatus,
    pub buck_status: BuckStatus,
}

impl From<u8> for SystemStatusResponse {
    fn from(value: u8) -> Self {
        Self {
            port_status: ((value & 0x02) >> 1).try_into().unwrap(),
            buck_status: (value & 0x01).try_into().unwrap(),
        }
    }
}

impl From<SystemStatusResponse> for u8 {
    fn from(value: SystemStatusResponse) -> Self {
        (value.port_status as u8) << 1 | (value.buck_status as u8)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum VinOvpStatus {
    Normal = 0,
    Ovp = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum OverTemperatureAlarmStatus {
    Normal = 0,
    Alarm = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum OverTemperatureShutdownStatus {
    Normal = 0,
    Shutdown = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum OutputShortCircuitStatus {
    Normal = 0,
    Short = 1,
}

#[derive(Debug)]
pub struct AbnormalCaseResponse {
    pub vin_ovp_status: VinOvpStatus,
    pub over_temperature_alarm_status: OverTemperatureAlarmStatus,
    pub over_temperature_shutdown_status: OverTemperatureShutdownStatus,
    pub output_short_circuit_status: OutputShortCircuitStatus,
}

impl From<u8> for AbnormalCaseResponse {
    fn from(value: u8) -> Self {
        Self {
            vin_ovp_status: ((value & 0x10) >> 4).try_into().unwrap(),
            over_temperature_alarm_status: ((value & 0x04) >> 2).try_into().unwrap(),
            over_temperature_shutdown_status: ((value & 0x02) >> 1).try_into().unwrap(),
            output_short_circuit_status: (value & 0x01).try_into().unwrap(),
        }
    }
}

impl From<AbnormalCaseResponse> for u8 {
    fn from(value: AbnormalCaseResponse) -> Self {
        (value.vin_ovp_status as u8) << 4
            | (value.over_temperature_alarm_status as u8) << 2
            | (value.over_temperature_shutdown_status as u8) << 1
            | (value.output_short_circuit_status as u8)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum BuckForceOff {
    Nothing = 0,
    TurnOffOneSecond = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum CCUnDrivenDurationBuckForceOff {
    Driven = 0,
    UnDriven = 1,
}

#[derive(Debug)]
pub struct BuckForceOffConfig {
    pub force_off: BuckForceOff,
    pub cc_un_driven_duration_buck_force_off: CCUnDrivenDurationBuckForceOff,
}

impl From<u8> for BuckForceOffConfig {
    fn from(value: u8) -> Self {
        Self {
            force_off: BuckForceOff::try_from((value & 0x80) >> 7).unwrap(),
            cc_un_driven_duration_buck_force_off: CCUnDrivenDurationBuckForceOff::try_from(
                (value & 0x40) >> 6,
            )
            .unwrap(),
        }
    }
}

impl From<BuckForceOffConfig> for u8 {
    fn from(value: BuckForceOffConfig) -> Self {
        (value.force_off as u8) << 7 | (value.cc_un_driven_duration_buck_force_off as u8) << 6
    }
}

#[derive(TryFromPrimitive, Debug, Clone, Copy)]
#[repr(u8)]
pub enum AdcConfig {
    Vin = 1,
    Vout = 2,
    Iout = 3,
}

#[derive(Debug)]
pub struct CcStatus {
    pub cc1_attached: bool,
    pub cc2_attached: bool,
}

impl From<u8> for CcStatus {
    fn from(value: u8) -> Self {
        Self {
            cc1_attached: (value & 0x80) != 0,
            cc2_attached: (value & 0x40) != 0,
        }
    }
}

impl From<CcStatus> for u8 {
    fn from(value: CcStatus) -> Self {
        (value.cc1_attached as u8) << 7 | (value.cc2_attached as u8) << 6
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PdCommand {
    HardReset = 1,
}

#[derive(Debug)]
pub struct PowerCommandRequest {
    /// PD source command send enable
    /// Write true, the command in (reg0x70[3:0]) will send. This bit is automatically cleared by hardware.
    pub send_enabled: bool,
    pub command: PdCommand,
}

impl TryFrom<u8> for PowerCommandRequest {
    type Error = ModelError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self {
            send_enabled: (value & 0x80) != 0,
            command: PdCommand::try_from(value & 0x07)
                .map_err(|_| ModelError::InvalidValueU8(value))?,
        })
    }
}

impl From<PowerCommandRequest> for u8 {
    fn from(value: PowerCommandRequest) -> Self {
        (value.send_enabled as u8) << 3 | (value.command as u8)
    }
}

impl Default for PowerCommandRequest {
    fn default() -> Self {
        Self {
            send_enabled: false,
            command: PdCommand::HardReset,
        }
    }
}

#[derive(Debug)]
pub struct FastChargeConfig6 {
    pub qc2_0_qc3_0_cable_compatible_and_offset_enabled: bool,
    pub pdo_link_with_vin: bool,
}

impl From<u8> for FastChargeConfig6 {
    fn from(value: u8) -> Self {
        Self {
            qc2_0_qc3_0_cable_compatible_and_offset_enabled: (value & 0x40) != 0,
            pdo_link_with_vin: (value & 0x20) != 0,
        }
    }
}

impl From<FastChargeConfig6> for u8 {
    fn from(value: FastChargeConfig6) -> Self {
        (value.qc2_0_qc3_0_cable_compatible_and_offset_enabled as u8) << 6
            | (value.pdo_link_with_vin as u8) << 5
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum ScpSelect {
    LowVoltage = 0,
    HighVoltage = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Pe2_0MaxVoltage {
    _12V = 0,
    _20V = 1,
}

#[derive(Debug)]
pub struct FastChargeConfig5 {
    pub scp_select: ScpSelect,
    pub pe2_0_max_voltage: Pe2_0MaxVoltage,
}

impl From<u8> for FastChargeConfig5 {
    fn from(value: u8) -> Self {
        Self {
            scp_select: ScpSelect::try_from((value & 0x40) >> 6).unwrap(),
            pe2_0_max_voltage: Pe2_0MaxVoltage::try_from((value & 0x20) >> 5).unwrap(),
        }
    }
}

impl From<FastChargeConfig5> for u8 {
    fn from(value: FastChargeConfig5) -> Self {
        (value.scp_select as u8) << 6 | (value.pe2_0_max_voltage as u8) << 5
    }
}

#[derive(Debug)]
pub struct FastChargeConfig0 {
    pub scp_enabled: bool,
    pub vooc_enabled: bool,
    pub sfcp_enabled: bool,
    pub qc2_0_enabled: bool,
    pub qc3_0_enabled: bool,
    pub fcp_enabled: bool,
    pub afc_enabled: bool,
    pub pe_enabled: bool,
}

impl From<u8> for FastChargeConfig0 {
    fn from(value: u8) -> Self {
        Self {
            scp_enabled: (value & 0x80) != 0,
            vooc_enabled: (value & 0x40) != 0,
            sfcp_enabled: (value & 0x20) != 0,
            qc2_0_enabled: (value & 0x10) != 0,
            qc3_0_enabled: (value & 0x08) != 0,
            fcp_enabled: (value & 0x04) != 0,
            afc_enabled: (value & 0x02) != 0,
            pe_enabled: (value & 0x01) != 0,
        }
    }
}

impl From<FastChargeConfig0> for u8 {
    fn from(value: FastChargeConfig0) -> Self {
        (value.scp_enabled as u8) << 7
            | (value.vooc_enabled as u8) << 6
            | (value.sfcp_enabled as u8) << 5
            | (value.qc2_0_enabled as u8) << 4
            | (value.qc3_0_enabled as u8) << 3
            | (value.fcp_enabled as u8) << 2
            | (value.afc_enabled as u8) << 1
            | (value.pe_enabled as u8)
    }
}

#[derive(Debug)]
pub struct FastChargeConfig1 {
    pub pps1_enabled: bool,
    pub pps0_enabled: bool,
    pub pd_20v_enabled: bool,
    pub pd_15v_enabled: bool,
    pub pd_12v_enabled: bool,
    pub pd_9v_enabled: bool,
    pub pd_enabled: bool,
}

impl From<u8> for FastChargeConfig1 {
    fn from(value: u8) -> Self {
        Self {
            pps1_enabled: (value & 0x80) != 0,
            pps0_enabled: (value & 0x40) != 0,
            pd_20v_enabled: (value & 0x20) != 0,
            pd_15v_enabled: (value & 0x10) != 0,
            pd_12v_enabled: (value & 0x08) != 0,
            pd_9v_enabled: (value & 0x04) != 0,
            pd_enabled: (value & 0x01) != 0,
        }
    }
}

impl From<FastChargeConfig1> for u8 {
    fn from(value: FastChargeConfig1) -> Self {
        (value.pps1_enabled as u8) << 7
            | (value.pps0_enabled as u8) << 6
            | (value.pd_20v_enabled as u8) << 5
            | (value.pd_15v_enabled as u8) << 4
            | (value.pd_12v_enabled as u8) << 3
            | (value.pd_9v_enabled as u8) << 2
            | (value.pd_enabled as u8)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum MaxOutputVoltageExceptPd {
    SameWithPd = 0,
    _9V = 1,
    _12V = 2,
    _20V = 3,
}
#[derive(Debug)]
pub struct FastChargeConfig2 {
    pub dpdm_enabled: bool,
    pub max_output_voltage_except_pd: MaxOutputVoltageExceptPd,
}

impl From<u8> for FastChargeConfig2 {
    fn from(value: u8) -> Self {
        Self {
            dpdm_enabled: (value & 0x20) != 0,
            max_output_voltage_except_pd: (value & 0x03).try_into().unwrap(),
        }
    }
}

impl From<FastChargeConfig2> for u8 {
    fn from(value: FastChargeConfig2) -> Self {
        (value.dpdm_enabled as u8) << 5 | (value.max_output_voltage_except_pd as u8)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PdCurrentLimitProtectMethod {
    /// UV mode, output voltage will go down when output current larger than current limit threshold
    UV = 0,
    /// OC mode, ouput voltage will reset to 5v when output current larger than current limit threshold
    OC = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum QC3_0CurrentLimitProtectMethod {
    CCLoop = 0,
    VoltageDrop = 1,
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum PowerSettingMethod {
    /// Rset resistance
    Rset = 0,
    /// Register reg0xA7
    Register = 1,
}

#[derive(Debug)]
pub struct FastChargeConfig3 {
    pub pd_current_limit_protect_method: PdCurrentLimitProtectMethod,
    pub qc3_0_current_limit_protect_method: QC3_0CurrentLimitProtectMethod,
    pub qc3_0_constant_power_enabled: bool,
    pub pps_constant_power_enabled: bool,
    pub power_setting_method: PowerSettingMethod,
}

impl From<u8> for FastChargeConfig3 {
    fn from(value: u8) -> Self {
        Self {
            pd_current_limit_protect_method: ((value & 0x80) >> 7).try_into().unwrap(),
            qc3_0_current_limit_protect_method: ((value & 0x40) >> 6).try_into().unwrap(),
            qc3_0_constant_power_enabled: (value & 0x20) != 0,
            pps_constant_power_enabled: (value & 0x10) != 0,
            power_setting_method: (value & 0x01).try_into().unwrap(),
        }
    }
}

impl From<FastChargeConfig3> for u8 {
    fn from(value: FastChargeConfig3) -> Self {
        (value.pd_current_limit_protect_method as u8) << 7
            | (value.qc3_0_current_limit_protect_method as u8) << 6
            | (value.qc3_0_constant_power_enabled as u8) << 5
            | (value.pps_constant_power_enabled as u8) << 4
            | (value.power_setting_method as u8) << 2
    }
}

#[derive(Debug)]
pub struct FastChargeConfig4 {
    /// if true, close all fast charge protocols
    pub port_fast_charge_disabled: bool,
    /// if true, source capability will be send again with 5V/2A PDO after device request 5V/3A PDO
    pub pd_5v_2a_rebroadcast_enabled: bool,
}

impl From<u8> for FastChargeConfig4 {
    fn from(value: u8) -> Self {
        Self {
            port_fast_charge_disabled: (value & 0x04) != 0,
            pd_5v_2a_rebroadcast_enabled: (value & 0x01) != 0,
        }
    }
}

impl From<FastChargeConfig4> for u8 {
    fn from(value: FastChargeConfig4) -> Self {
        (value.port_fast_charge_disabled as u8) << 2 | (value.pd_5v_2a_rebroadcast_enabled as u8)
    }
}
