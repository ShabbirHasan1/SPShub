#[allow(unused_imports)]
use super::compass_data::{CompassData, decompose_uuid_to_board_channel};
use super::channel_map::{ChannelMap, SPSChannelType};
use super::used_size::UsedSize;
use std::collections::BTreeMap;
use std::hash::Hash;

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumCount, AsRefStr};

use polars::prelude::*;

const INVALID_VALUE: f64 = -1.0e6;

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord, PartialEq, EnumIter, EnumCount, AsRefStr)]
pub enum SPSDataField {
    AnodeFrontEnergy,
    AnodeFrontShort,
    AnodeFrontTime,
    AnodeBackEnergy,
    AnodeBackShort,
    AnodeBackTime,
    ScintLeftEnergy,
    ScintLeftShort,
    ScintLeftTime,
    ScintRightEnergy,
    ScintRightShort,
    ScintRightTime,
    CathodeEnergy,
    CathodeShort,
    CathodeTime,
    DelayFrontLeftEnergy,
    DelayFrontLeftShort,
    DelayFrontLeftTime,
    DelayFrontRightEnergy,
    DelayFrontRightShort,
    DelayFrontRightTime,
    DelayBackLeftEnergy,
    DelayBackLeftShort,
    DelayBackLeftTime,
    DelayBackRightEnergy,
    DelayBackRightShort,
    DelayBackRightTime,
    X1,
    X2,
    Xavg,
    Theta,

    Cebra0Energy,
    Cebra1Energy,
    Cebra2Energy,
    Cebra3Energy,
    Cebra4Energy,
    Cebra5Energy,
    Cebra6Energy,

    Cebra0Short,
    Cebra1Short,
    Cebra2Short,
    Cebra3Short,
    Cebra4Short,
    Cebra5Short,
    Cebra6Short,

    Cebra0Time,
    Cebra1Time,
    Cebra2Time,
    Cebra3Time,
    Cebra4Time,
    Cebra5Time,
    Cebra6Time,

}

impl SPSDataField {
    //Returns a list of fields for iterating over
    pub fn get_field_vec() -> Vec<SPSDataField> {
        SPSDataField::iter().collect()
    }
}

impl UsedSize for SPSDataField {
    fn get_used_size(&self) -> usize {
        std::mem::size_of::<SPSDataField>()
    }
}

#[derive(Debug, Clone)]
pub struct SPSData {
    //Columns must always come in same order, so use sorted map
    pub fields: BTreeMap<SPSDataField, Vec<f64>>,
    pub rows: usize
}

impl Default for SPSData {
    fn default() -> Self {
        let fields = SPSDataField::get_field_vec();
        let mut data = SPSData { fields: BTreeMap::new(), rows: 0 };
        fields.into_iter().for_each(|f| { data.fields.insert(f, vec![]); });
        return data;
    }
}

impl UsedSize for SPSData {
    fn get_used_size(&self) -> usize {
        self.fields.get_used_size()
    }
}

impl SPSData {

    //To keep columns all same length, push invalid values as necessary
    fn push_defaults(&mut self) {
        for field in self.fields.iter_mut() {
            if field.1.len() < self.rows {
                field.1.push(INVALID_VALUE)
            }
        }
    }

    //Update the last element to the given value
    fn set_value(&mut self, field: &SPSDataField, value: f64) {
        if let Some(list) = self.fields.get_mut(field) {
            if let Some(back) = list.last_mut() {
                *back = value;
            }
        }
    }

    pub fn append_event(&mut self, event: Vec<CompassData>, map: &ChannelMap, weights: Option<(f64, f64)>) {

        self.rows += 1;
        self.push_defaults();


        let mut dfl_time = INVALID_VALUE;
        let mut dfr_time = INVALID_VALUE;
        let mut dbl_time = INVALID_VALUE;
        let mut dbr_time = INVALID_VALUE;
        
        for hit in event.iter() {
            //Fill out detector fields using channel map
            let channel_data = match map.get_channel_data(&hit.uuid) {
                Some(data) => data,
                None => continue
            };
            match channel_data.channel_type {
                SPSChannelType::ScintLeft => {
                    self.set_value(&SPSDataField::ScintLeftEnergy, hit.energy);
                    self.set_value(&SPSDataField::ScintLeftShort, hit.energy_short);
                    self.set_value(&SPSDataField::ScintLeftTime, hit.timestamp);
                }

                SPSChannelType::ScintRight => {
                    self.set_value(&SPSDataField::ScintRightEnergy, hit.energy);
                    self.set_value(&SPSDataField::ScintRightShort, hit.energy_short);
                    self.set_value(&SPSDataField::ScintRightTime, hit.timestamp);
                }

                SPSChannelType::Cathode => {
                    self.set_value(&SPSDataField::CathodeEnergy, hit.energy);
                    self.set_value(&SPSDataField::CathodeShort, hit.energy_short);
                    self.set_value(&SPSDataField::CathodeTime, hit.timestamp);
                }

                SPSChannelType::DelayFrontRight => {
                    self.set_value(&SPSDataField::DelayFrontRightEnergy, hit.energy);
                    self.set_value(&SPSDataField::DelayFrontRightShort, hit.energy_short);
                    self.set_value(&SPSDataField::DelayFrontRightTime, hit.timestamp);
                    dfr_time = hit.timestamp;
                }

                SPSChannelType::DelayFrontLeft => {
                    self.set_value(&SPSDataField::DelayFrontLeftEnergy, hit.energy);
                    self.set_value(&SPSDataField::DelayFrontLeftShort, hit.energy_short);
                    self.set_value(&SPSDataField::DelayFrontLeftTime, hit.timestamp);
                    dfl_time = hit.timestamp;
                }

                SPSChannelType::DelayBackRight => {
                    self.set_value(&SPSDataField::DelayBackRightEnergy, hit.energy);
                    self.set_value(&SPSDataField::DelayBackRightShort, hit.energy_short);
                    self.set_value(&SPSDataField::DelayBackRightTime, hit.timestamp);
                    dbr_time = hit.timestamp;
                }

                SPSChannelType::DelayBackLeft => {
                    self.set_value(&SPSDataField::DelayBackLeftEnergy, hit.energy);
                    self.set_value(&SPSDataField::DelayBackLeftShort, hit.energy_short);
                    self.set_value(&SPSDataField::DelayBackLeftTime, hit.timestamp);
                    dbl_time = hit.timestamp;
                }

                SPSChannelType::AnodeFront => {
                    self.set_value(&SPSDataField::AnodeFrontEnergy, hit.energy);
                    self.set_value(&SPSDataField::AnodeFrontShort, hit.energy_short);
                    self.set_value(&SPSDataField::AnodeFrontTime, hit.timestamp);
                }

                SPSChannelType::AnodeBack => {
                    self.set_value(&SPSDataField::AnodeBackEnergy, hit.energy);
                    self.set_value(&SPSDataField::AnodeBackShort, hit.energy_short);
                    self.set_value(&SPSDataField::AnodeBackTime, hit.timestamp);
                }

                SPSChannelType::Cebra0 => {
                    self.set_value(&SPSDataField::Cebra0Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra0Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra0Time, hit.timestamp);
                }

                SPSChannelType::Cebra1 => {
                    self.set_value(&SPSDataField::Cebra1Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra1Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra1Time, hit.timestamp);
                }

                SPSChannelType::Cebra2 => {
                    self.set_value(&SPSDataField::Cebra2Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra2Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra2Time, hit.timestamp);
                }

                SPSChannelType::Cebra3 => {
                    self.set_value(&SPSDataField::Cebra3Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra3Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra3Time, hit.timestamp);
                }

                SPSChannelType::Cebra4 => {
                    self.set_value(&SPSDataField::Cebra4Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra4Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra4Time, hit.timestamp);
                }

                SPSChannelType::Cebra5 => {
                    self.set_value(&SPSDataField::Cebra5Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra5Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra5Time, hit.timestamp);
                }

                SPSChannelType::Cebra6 => {
                    self.set_value(&SPSDataField::Cebra6Energy, hit.energy);
                    self.set_value(&SPSDataField::Cebra6Short, hit.energy_short);
                    self.set_value(&SPSDataField::Cebra6Time, hit.timestamp);
                }


                _ =>  continue
            }
        }

        //Physics
        let mut x1 = INVALID_VALUE;
        let mut x2 = INVALID_VALUE;
        if dfr_time != INVALID_VALUE && dfl_time != INVALID_VALUE {
            x1 = (dfl_time - dfr_time) * 0.5 * 1.0/2.1;
            self.set_value(&SPSDataField::X1, x1);
        }
        if dbr_time != INVALID_VALUE && dbl_time != INVALID_VALUE {
            x2 = (dbl_time - dbr_time) * 0.5 * 1.0/1.98;
            self.set_value(&SPSDataField::X2, x2);
        }
        if x1 != INVALID_VALUE && x2 != INVALID_VALUE {
            let diff = x2 -x1;
            if diff > 0.0 {
                self.set_value(&SPSDataField::Theta, (diff/36.0).atan());
            } else if diff < 0.0 {
                self.set_value(&SPSDataField::Theta, std::f64::consts::PI + (diff/36.0).atan());
            } else {
                self.set_value(&SPSDataField::Theta, std::f64::consts::PI * 0.5);
            }

            match weights {
               Some(w) => self.set_value(&SPSDataField::Xavg, w.0 * x1 + w.1 * x2),
               None => self.set_value(&SPSDataField::Xavg, INVALID_VALUE)
            };
        }

    }

    pub fn convert_to_series(self) -> Vec<Series> {
        let sps_cols: Vec<Series> = self.fields.into_iter()
                    .map(|field| -> Series {
                        Series::new(field.0.as_ref(), field.1)
                    })
                    .collect();

        return sps_cols
    }
}