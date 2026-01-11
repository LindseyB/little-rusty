// Audio module using kira for reliable audio playback
use kira::{
    manager::{AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    Volume,
};

pub struct AudioSystem {
    manager: AudioManager,
}

impl AudioSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = AudioManager::new(AudioManagerSettings::default())?;
        println!("🎵 Kira audio system initialized");
        Ok(AudioSystem { manager })
    }
    
    pub fn play_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sound_data = StaticSoundData::from_file(file_path)?;
        let sound = sound_data.with_settings(StaticSoundSettings::new().volume(Volume::Amplitude(0.5)));
        self.manager.play(sound)?;
        println!("🎵 Playing audio file: {}", file_path);
        Ok(())
    }
    
    pub fn play_file_looped(&mut self, file_path: &str, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        let sound_data = StaticSoundData::from_file(file_path)?;
        let sound = sound_data.with_settings(
            StaticSoundSettings::new()
                .volume(Volume::Amplitude(volume as f64))
                .loop_region(..) // Loop the entire sound
        );
        self.manager.play(sound)?;
        println!("🔄 Playing audio file on loop: {} (volume: {:.1}%)", file_path, volume * 100.0);
        Ok(())
    }
    
    pub fn set_volume(&self, volume: f32) {
        println!("🔊 Note: Volume is set per-sound in kira (currently {:.1}%)", volume * 100.0);
    }
}