pub const ICONS: &[u8] = include_bytes!("../assets/crc-icons.ttf");

pub const SANS: [&[u8]; 3] = [
    include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSans-Medium.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf"),
];

pub const MONO: [&[u8]; 2] = [
    include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-SemiBold.ttf"),
];

pub fn all() -> Vec<&'static [u8]> {
    let mut faces = vec![ICONS];
    faces.extend(SANS);
    faces.extend(MONO);
    faces
}
