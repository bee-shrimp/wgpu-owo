use rodio::Source;
use rodio::source::SineWave;
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

    // pub fn play_sine(&self) -> anyhow::Result<()> {
    //     let player = rodio::Player::connect_new(&self.stream_handle.mixer());
    //
    //     let source = SineWave::new(700.0)
    //         .take_duration(Duration::from_secs_f32(0.2))
    //         .amplify(0.20);
    //     player.append(source);
    //
    //     thread::sleep(Duration::from_millis(1500));
    //     Ok(())
    // }

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
}
