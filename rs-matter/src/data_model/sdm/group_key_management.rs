/*
 *
 *    Copyright (c) 2023 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use core::convert::TryInto;

use crate::{
    attribute_enum, cmd_enter, command_enum,
    data_model::objects::AttrType,
    data_model::objects::*,
    error::{Error, ErrorCode},
    tlv::TLVElement,
    transport::exchange::Exchange,
    utils::rand::Rand,
};
use log::info;
use strum::{EnumDiscriminants, FromRepr};

pub const ID: u32 = 0x003F;

#[derive(FromRepr, EnumDiscriminants)]
#[repr(u16)]
pub enum Attributes {
    GroupKeyMap(()) = 0x00,
    GroupTable(()) = 0x01,
    MaxGroupsPerFabric(AttrType<u16>) = 0x02,
    MaxGroupKeysPerFabric(AttrType<u16>) = 0x03,
}

attribute_enum!(Attributes);

#[derive(FromRepr, EnumDiscriminants)]
#[repr(u32)]
pub enum Commands {
    KeySetWrite = 0x0,
}

command_enum!(Commands);

pub const CLUSTER: Cluster<'static> = Cluster {
    id: ID as _,
    feature_map: 0,
    attributes: &[
        FEATURE_MAP,
        ATTRIBUTE_LIST,
        Attribute::new(
            AttributesDiscriminants::GroupKeyMap as u16,
            Access::RWFVM,
            Quality::PERSISTENT,
        ),
        Attribute::new(
            AttributesDiscriminants::GroupTable as u16,
            Access::RF,
            Quality::NONE,
        ),
        Attribute::new(
            AttributesDiscriminants::MaxGroupsPerFabric as u16,
            Access::READ,
            Quality::FIXED,
        ),
        Attribute::new(
            AttributesDiscriminants::MaxGroupKeysPerFabric as u16,
            Access::READ,
            Quality::FIXED,
        ),
    ],
    commands: &[CommandsDiscriminants::KeySetWrite as _],
};

pub struct GrpKeyMgmtCluster {
    data_ver: Dataver,
}

impl GrpKeyMgmtCluster {
    pub fn new(rand: Rand) -> Self {
        Self {
            data_ver: Dataver::new(rand),
        }
    }

    pub fn read(&self, attr: &AttrDetails, encoder: AttrDataEncoder) -> Result<(), Error> {
        if let Some(writer) = encoder.with_dataver(self.data_ver.get())? {
            if attr.is_system() {
                CLUSTER.read(attr.attr_id, writer)
            } else {
                match attr.attr_id.try_into()? {
                    Attributes::MaxGroupsPerFabric(codec) => codec.encode(writer, 1),
                    Attributes::MaxGroupKeysPerFabric(codec) => codec.encode(writer, 1),
                    _ => Err(ErrorCode::AttributeNotFound.into()),
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn write(&self, _attr: &AttrDetails, data: AttrData) -> Result<(), Error> {
        let _data = data.with_dataver(self.data_ver.get())?;

        self.data_ver.changed();

        Ok(())
    }

    pub fn invoke(
        &self,
        _exchange: &Exchange,
        cmd: &CmdDetails,
        _data: &TLVElement,
        _encoder: CmdDataEncoder,
    ) -> Result<(), Error> {
        match cmd.cmd_id.try_into()? {
            Commands::KeySetWrite => {
                cmd_enter!("KeySetWrite: Not yet supported");
            }
        }

        self.data_ver.changed();

        Ok(())
    }
}

impl Handler for GrpKeyMgmtCluster {
    fn read(&self, attr: &AttrDetails, encoder: AttrDataEncoder) -> Result<(), Error> {
        GrpKeyMgmtCluster::read(self, attr, encoder)
    }

    fn write(&self, attr: &AttrDetails, data: AttrData) -> Result<(), Error> {
        GrpKeyMgmtCluster::write(self, attr, data)
    }

    fn invoke(
        &self,
        exchange: &Exchange,
        cmd: &CmdDetails,
        data: &TLVElement,
        encoder: CmdDataEncoder,
    ) -> Result<(), Error> {
        GrpKeyMgmtCluster::invoke(self, exchange, cmd, data, encoder)
    }
}

// TODO: Might be removed once the `on` member is externalized
impl NonBlockingHandler for GrpKeyMgmtCluster {}

impl ChangeNotifier<()> for GrpKeyMgmtCluster {
    fn consume_change(&mut self) -> Option<()> {
        self.data_ver.consume_change(())
    }
}
