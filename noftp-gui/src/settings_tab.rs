use crate::WarnErr;

pub struct EditingIpTab {
    pub ip: String,
    pub ip_alias: String,
}

pub struct FriendIpTab {
    pub ip: String,
    pub ip_alias: String,
    pub editing: EditingIpTab
}

pub struct SettingsTab {
    pub port: String,
    pub friend_ip: FriendIpTab,
    pub(crate) message: Option<WarnErr>,
    pub download_path: String
}
