use std::{io::Cursor, sync::Arc};

use anyhow::Result;
use rodio::{Decoder, Source};

pub struct SoundPlayer {
    _stream_handle: rodio::MixerDeviceSink,
    _bgm_player: rodio::Player,
    mixer: rodio::mixer::Mixer,
    sources: [Arc<[u8]>; 2],
}

impl SoundPlayer {
    pub async fn new() -> Result<Self> {
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        let mixer = stream_handle.mixer().clone();

        let spawn = Arc::from(async_fs::read("assets/powerUp.wav").await?);
        let despawn = Arc::from(async_fs::read("assets/explosion.wav").await?);

        let bgm_bytes = async_fs::read("assets/crow_song.wav").await?;
        let source = rodio::Decoder::try_from(Cursor::new(bgm_bytes))?.repeat_infinite();

        let player = rodio::Player::connect_new(stream_handle.mixer());
        player.append(source);

        Ok(Self {
            _stream_handle: stream_handle,
            _bgm_player: player,
            mixer,
            sources: [spawn, despawn],
        })
    }

    pub fn play_spawn(&self) -> Result<()> {
        let cursor = Cursor::new(self.sources[0].clone());
        let source = Decoder::try_from(cursor)?.amplify(0.5);
        self.mixer.add(source);
        Ok(())
    }

    pub fn play_despawn(&self) -> Result<()> {
        let cursor = Cursor::new(self.sources[1].clone());
        let source = Decoder::try_from(cursor)?.amplify(0.5);
        self.mixer.add(source);

        Ok(())
    }
}
