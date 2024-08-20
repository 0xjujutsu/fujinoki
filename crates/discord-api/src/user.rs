use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize};
use turbopack_binding::turbo::{
    tasks as turbo_tasks,
    tasks::{RcStr, TaskInput},
};

use crate::id::UserId;

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Serialize, Deserialize, Debug, TaskInput, Hash)]
pub struct User {
    /// the user's id
    pub id: UserId,
    /// the user's username, not unique across the platform
    pub username: RcStr,
    /// the user's Discord-tag
    pub discriminator: RcStr,
    /// the user's display name, if it is set. For bots, this is the application
    /// name
    pub global_name: Option<RcStr>,
    /// the user's avatar hash
    pub avatar: Option<RcStr>,
    /// whether the user belongs to an OAuth2 application
    pub bot: Option<bool>,
    /// whether the user is an Official Discord System user (part of the urgent
    /// message system)
    pub system: Option<bool>,
    /// whether the user has two factor enabled on their account
    pub mfa_enabled: Option<bool>,
    /// the user's banner hash
    pub banner: Option<RcStr>,
    /// the user's banner color encoded as an integer representation of
    /// hexadecimal color code
    pub accent_color: Option<u32>,
    /// the user's chosen language option
    pub locale: Option<RcStr>,
    /// whether the email on this account has been verified
    pub verified: Option<bool>,
    /// the user's email
    pub email: Option<RcStr>,
    /// the flags on a user's account
    pub flags: Option<UserFlags>,
    /// the type of Nitro subscription on a user's account
    pub premium_type: Option<PremiumType>,
    /// the public flags on a user's account
    pub public_flags: Option<u32>,
    /// the user's avatar decoration hash
    pub avatar_decoration: Option<RcStr>,
}

bitflags! {
    #[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
    #[derive(Serialize, Default, TaskInput)]
    pub struct UserFlags: u32 {
        /// Discord Employee
        const STAFF = 1<<0;
        /// Partnered Server Owner
        const PARTNER = 1<<1;
        /// HypeSquad Events Member
        const HYPESQUAD = 1<<2;
        /// Bug Hunter Level 1
        const BUG_HUNTER_LEVEL_1 = 1<<3;
        /// House Bravery Member
        const HYPESQUAD_ONLINE_HOUSE_1 = 1<<6;
        /// House Brilliance Member
        const HYPESQUAD_ONLINE_HOUSE_2 = 1<<7;
        /// House Balance Member
        const HYPESQUAD_ONLINE_HOUSE_3 = 1<<8;
        /// Early Nitro Supporter
        const PREMIUM_EARLY_SUPPORTER = 1<<9;
        /// User is a team
        const TEAM_PSEUDO_USER = 1<<10;
        /// Bug Hunter Level 2
        const BUG_HUNTER_LEVEL_2 = 1<<14;
        /// Verified Bot
        const VERIFIED_BOT = 1<<16;
        /// Early Verified Bot Developer
        const VERIFIED_DEVELOPER = 1<<17;
        /// Moderator Programs Alumni
        const CERTIFIED_MODERATOR = 1<<18;
        /// Bot uses only HTTP interactions and is shown in the online member list
        const BOT_HTTP_INTERACTIONS = 1<<19;
        /// User is an Active Developer
        const ACTIVE_DEVELOPER = 1<<22;
    }
}

impl<'de> Deserialize<'de> for UserFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(UserFlags::from_bits_truncate(value))
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Serialize, Debug, TaskInput, Hash)]
pub enum PremiumType {
    None = 0,
    NitroClassic = 1,
    Nitro = 2,
    NitroBasic = 3,
}

impl<'de> Deserialize<'de> for PremiumType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(match value {
            0 => Self::None,
            1 => Self::NitroClassic,
            2 => Self::Nitro,
            3 => Self::NitroBasic,
            _ => Self::None,
        })
    }
}
