use napi::bindgen_prelude::{BigInt, Buffer};
use napi_derive::napi;
use steamworks::SteamId;

#[derive(Debug)]
#[napi(object)]
pub struct PlayerSteamId {
    pub steam_id64: BigInt,
    pub steam_id32: String,
    pub account_id: u32,
}

/// The local player's avatar as RGBA pixels, row by row from the top left.
#[napi(object)]
pub struct PlayerAvatar {
    pub width: u32,
    pub height: u32,
    pub rgba: Buffer,
}

impl PlayerSteamId {
    pub(crate) fn from_steamid(steam_id: SteamId) -> Self {
        Self {
            steam_id64: steam_id.raw().into(),
            steam_id32: steam_id.steamid32(),
            account_id: steam_id.account_id().raw(),
        }
    }
}

#[napi]
pub mod localplayer {
    use super::{PlayerAvatar, PlayerSteamId};

    #[napi]
    pub fn get_steam_id() -> PlayerSteamId {
        let client = crate::client::get_client();
        let steam_id = client.user().steam_id();
        PlayerSteamId::from_steamid(steam_id)
    }

    #[napi]
    pub fn get_name() -> String {
        let client = crate::client::get_client();
        client.friends().name()
    }

    #[napi]
    pub fn get_level() -> u32 {
        let client = crate::client::get_client();
        client.user().level()
    }

    /// @returns the 2 digit ISO 3166-1-alpha-2 format country code which client is running in, e.g. "US" or "UK".
    #[napi]
    pub fn get_ip_country() -> String {
        let client = crate::client::get_client();
        client.utils().ip_country()
    }

    /// @returns the player's large avatar, or null when the player has none or Steam is still loading it
    #[napi]
    pub fn get_avatar() -> Option<PlayerAvatar> {
        // Steam's large avatar is 184 pixels square. A side above this cap is
        // a malformed image, rejected before the pixel buffer is allocated;
        // Setter's Electron facade applies the same cap.
        const MAX_SIDE: u32 = 512;
        let client = crate::client::get_client();
        let steam_id = client.user().steam_id();
        unsafe {
            let friends = steamworks::sys::SteamAPI_SteamFriends_v017();
            let utils = steamworks::sys::SteamAPI_SteamUtils_v010();
            // 0: no avatar set; -1: the image is still being loaded.
            let image = steamworks::sys::SteamAPI_ISteamFriends_GetLargeFriendAvatar(
                friends,
                steam_id.raw(),
            );
            if image <= 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !steamworks::sys::SteamAPI_ISteamUtils_GetImageSize(
                utils,
                image,
                &mut width,
                &mut height,
            ) {
                return None;
            }
            if !(1..=MAX_SIDE).contains(&width) || !(1..=MAX_SIDE).contains(&height) {
                return None;
            }
            let size = width as usize * height as usize * 4;
            let mut rgba = vec![0; size];
            if !steamworks::sys::SteamAPI_ISteamUtils_GetImageRGBA(
                utils,
                image,
                rgba.as_mut_ptr(),
                size as i32,
            ) {
                return None;
            }
            Some(PlayerAvatar {
                width,
                height,
                rgba: rgba.into(),
            })
        }
    }

    #[napi]
    pub fn set_rich_presence(key: String, value: Option<String>) {
        let client = crate::client::get_client();
        client.friends().set_rich_presence(&key, value.as_deref());
    }
}
