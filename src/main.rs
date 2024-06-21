#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
mod api;
mod wav;
use std::{
    ffi::CString,
    fs,
    io::{self, stdout},
};

use api::{AccelerationMode, VoicevoxCore};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{error, info, warn};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, List, ListState,
    },
    Terminal,
};
use ratatui_explorer::{FileExplorer, Theme};
use regex::Regex;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Speaker {
    name: String,
    styles: Vec<SpeakerStyle>,
    speaker_uuid: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct SpeakerStyle {
    name: String,
    id: u32,
    #[serde(rename = "type")]
    _type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AudioQueryResult {
    accent_phrases: Vec<AccentPhrases>,
    speed_scale: f32,
    pitch_scale: f32,
    intonation_scale: f32,
    volume_scale: f32,
    pre_phoneme_length: f32,
    post_phoneme_length: f32,
    output_sampling_rate: i32,
    output_stereo: bool,
    kana: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccentPhrases {
    accent: i32,
    is_interrogative: bool,
    moras: Vec<Mora>,
    pause_mora: Option<Mora>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mora {
    consonant: Option<String>,
    consonant_length: Option<f32>,
    pitch: f32,
    text: String,
    vowel: String,
    vowel_length: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Loading all speakers...");
    let speakers = std::fs::File::open("./model/metas.json").expect("Speaker data file not found");
    let speakers: Vec<Speaker> =
        serde_json::from_reader(speakers).expect("Failed to parse speaker data");

    let speakers: Vec<String> = speakers
        .iter()
        .map(|speaker| {
            speaker
                .styles
                .iter()
                .filter(|style| style._type.is_none())
                .map(|style| speaker.name.clone() + " " + &style.name + "-" + &style.id.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect();

    let dir = CString::new("./assets/open_jtalk_dic_utf_8-1.11").unwrap();
    let vvc =
        VoicevoxCore::new_from_options(AccelerationMode::Auto, 10, false, dir.as_c_str()).unwrap();

    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut speaker_select_state = ListState::default();
    speaker_select_state.select(Some(0));
    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let instructions = Title::from(Line::from(vec![
                " 移动 ".into(),
                "<↑/↓>".blue().bold(),
                " 选择 ".into(),
                "<Enter>".blue().bold(),
                " 退出 ".into(),
                "<Q> ".blue().bold(),
            ]));
            let list = List::new(
                speakers
                    .iter()
                    .map(|speaker| speaker.split_once("-").unwrap().0.to_string())
                    .collect::<Vec<String>>(),
            )
            .block(
                Block::bordered()
                    .title(
                        Title::from("选择声音".red().bold())
                            .position(Position::Bottom)
                            .alignment(Alignment::Center),
                    )
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    ),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
            frame.render_stateful_widget(list, area, &mut speaker_select_state);
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            speaker_select_state
                                .select(Some(speaker_select_state.selected().unwrap().max(1) - 1));
                        }
                        KeyCode::Down => {
                            speaker_select_state.select(
                                Some(speaker_select_state.selected().unwrap() + 1)
                                    .min(Some(speakers.len() - 1)),
                            );
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let theme = Theme::default()
        .with_title_bottom(|file_explorer: &FileExplorer| {
            Line::from(vec![
                " 选择文本文本文件 ".red().bold(),
                " 进入目录 ".into(),
                " <←/→> ".blue().bold(),
                " 当前目录 ".into(),
                file_explorer.cwd().display().to_string().blue().bold(),
            ])
            .alignment(Alignment::Center)
        })
        .with_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .with_highlight_item_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_dir_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .with_highlight_symbol("> ".into());
    let mut file_explorer = FileExplorer::with_theme(theme)?;

    loop {
        terminal.draw(|frame| {
            frame.render_widget(&file_explorer.widget(), frame.size());
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;

            if let event::Event::Key(key) = event {
                if KeyEventKind::Press == key.kind {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            file_explorer.handle(&event)?;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    let txt_path = file_explorer.current().path().to_str().unwrap().to_string();
    info!("Selected file {}", txt_path);
    if !txt_path.ends_with(".txt") {
        error!("Selected file must be .txt");
        return Ok(());
    }

    let mut speed_scale;
    loop {
        info!("Please enter speed scale (a float number greater than 0): ");
        let mut speed_scale_str = String::new();
        io::stdin().read_line(&mut speed_scale_str)?;
        speed_scale = speed_scale_str.trim().parse::<f32>().unwrap();
        info!("Speed Scale: {}", speed_scale);
        if speed_scale > 0.0 {
            break;
        }
        error!("Invalid speed scale, please enter again");
    }

    let speaker_id = speaker_select_state.selected().unwrap() as u32;
    info!("Speaker ID: {}", speaker_id);
    info!("Loading model...");
    vvc.load_model(speaker_id).unwrap();

    info!("Model load completed");

    let mut lines = Vec::new();
    for txt in fs::read_to_string(txt_path)?.lines() {
        lines.push(txt.to_string());
    }
    let total_line = lines.len();
    info!("Loaded {} lines", total_line);
    info!("Start TTS...");
    let mut datas: Vec<Vec<u8>> = Vec::new();
    let mut synthesis_option = VoicevoxCore::make_default_synthesis_options();
    let mut query_option = VoicevoxCore::make_default_audio_query_options();
    query_option.kana = false;
    synthesis_option.enable_interrogative_upspeak = true;

    let mut line_count = 0;
    for line in lines {
        line_count += 1;
        let line = line.trim();
        if line.len() == 0 {
            warn!("Empty line, skipping");
            continue;
        }
        let regex = Regex::new("[一-龠ぁ-ゔァ-ヴーａ-ｚＡ-Ｚ０-９々〆〤]+").unwrap();
        let captures = regex.captures(line);
        if captures.is_none() {
            warn!(
                "Line {} does not contain any valid japanese character, skipping",
                line
            );
            continue;
        }
        info!("Processing {}", line);

        // Do full width char replacement
        let regex = Regex::new("[。·ˉˇ¨〃々—～‖…‘’“”〔〕〈〉《》「」『』〖〗【】！＂＇（）〇◯○，－．：；＜＝＞［］｛｜｝｀﹉﹊﹋﹌﹍﹎﹏﹐﹑﹒﹔﹕﹖﹗﹙﹚﹛﹜﹝﹞︵︶︹︺︿﹀︽︾﹁﹂﹃﹄︻︼︷︸︱︳︴]").unwrap();
        let mut line = regex.replace_all(line, "、").to_string();
        if line.starts_with("、") {
            line = line.trim_start_matches('、').to_string();
        }
        let audio_query = vvc.audio_query(&line, speaker_id, query_option).unwrap();
        let mut audio_query: AudioQueryResult = serde_json::from_str(audio_query.as_str()).unwrap();
        if audio_query
            .accent_phrases
            .last()
            .unwrap()
            .pause_mora
            .is_none()
        {
            audio_query.accent_phrases.last_mut().unwrap().pause_mora = Some(Mora {
                consonant: None,
                consonant_length: None,
                pitch: 0.,
                text: "、".to_string(),
                vowel: "pau".to_string(),
                vowel_length: 0.5,
            });
        }
        audio_query.speed_scale = speed_scale;
        let audio_query: String = serde_json::to_string(&audio_query).unwrap();
        let wav = vvc
            .synthesis(audio_query.as_str(), speaker_id, synthesis_option)
            .unwrap();
        datas.push(wav.as_slice().to_vec());
        info!("Processed {} / {} line", line_count, total_line);
    }

    info!("Concating audio...");
    wav::wav_concat(datas, "out_merge.wav".to_string());
    info!("Done! Press any key to exit");
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}
