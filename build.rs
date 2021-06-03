extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILEVERSION, 1 << 48);
        res.compile().unwrap();
    }
}