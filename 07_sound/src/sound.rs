use std::io::Cursor;

use rodio::{Decoder, Source};

pub struct SoundPlayer {
    _stream_handle: rodio::MixerDeviceSink,
    _bgm_player: rodio::Player,
    mixer: rodio::mixer::Mixer,
    sources: [Vec<u8>; 2],
}

impl SoundPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        let mixer = stream_handle.mixer().clone();

        let spawn = std::fs::read("assets/powerUp.wav")?;
        let despawn = std::fs::read("assets/explosion.wav")?;

        let player = rodio::Player::connect_new(stream_handle.mixer());

        let file = std::fs::File::open("assets/crow_song.wav")?;
        let source = rodio::Decoder::try_from(file)?.repeat_infinite();
        player.append(source);

        Ok(Self {
            _stream_handle: stream_handle,
            _bgm_player: player,
            mixer,
            sources: [spawn, despawn],
        })
    }

    pub fn play_spawn(&self) -> anyhow::Result<()> {
        let cursor = Cursor::new(self.sources[0].clone());
        let source = Decoder::try_from(cursor)?.amplify(0.5);
        self.mixer.add(source);

        Ok(())
    }

    pub fn play_despawn(&self) -> anyhow::Result<()> {
        let cursor = Cursor::new(self.sources[1].clone());
        let source = Decoder::try_from(cursor)?.amplify(0.5);
        self.mixer.add(source);

        Ok(())
    }
}
