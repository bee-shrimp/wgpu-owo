use std::io::BufReader;
use std::thread;
use std::time::Duration;

pub struct SoundPlayer {
    stream_handle: rodio::MixerDeviceSink,
}

impl SoundPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        Ok(Self { stream_handle })
    }

    pub fn play_despawn(&self) -> anyhow::Result<()> {
        let mixer = self.stream_handle.mixer();

        let despawn = {
            // Play a WAV file.
            let file = std::fs::File::open("assets/explosion.wav")?;
            let player = rodio::play(mixer, BufReader::new(file))?;
            player.set_volume(0.2);
            player
        };
        println!("Started despawn (explosion.wav)");
        thread::sleep(Duration::from_millis(1500));

        drop(despawn);
        println!("Stopped despawn");

        Ok(())
    }

    pub fn play_spawn(&self) -> anyhow::Result<()> {
        let mixer = self.stream_handle.mixer();

        let spawn = {
            // Play a WAV file.
            let file = std::fs::File::open("assets/powerUp.wav")?;
            let player = rodio::play(mixer, BufReader::new(file))?;
            player.set_volume(0.2);
            player
        };
        println!("Started spawn (explosion.wav)");
        thread::sleep(Duration::from_millis(1500));

        drop(spawn);
        println!("Stopped spawn");

        Ok(())
    }
}
