use windows_volume_control::AudioController;

pub struct SessionContainer {
    pub controller: AudioController,
    pub sessionName: String,
    pub initialVolume: f32
}