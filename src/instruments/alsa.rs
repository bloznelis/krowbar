use alsa::mixer::{Mixer, Selem, SelemChannelId, SelemId};
use anyhow::anyhow;

pub fn get_volume_percents() -> anyhow::Result<f64> {
    let mixer = Mixer::new("default", false)?;
    let selem: &Selem = &mixer
        .find_selem(&SelemId::new("Master", 0))
        .ok_or(anyhow!("Failed to find master selem."))?;

    let (_, max) = selem.get_playback_volume_range();
    // XXX: presume that left right channels have the same volume
    let volume = selem.get_playback_volume(SelemChannelId::FrontLeft)?;

    Ok((volume as f64 / max as f64) * 100.0)
}
