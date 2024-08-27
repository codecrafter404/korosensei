# Koro-sensei
> Supercharge your school notes at mach 20

## What?
- This programm will help you organise your notes, by automating the following things

## Functions (TODO)
- [ ] GitHub actions
- [x] automatically transcribe your audio files and attach them to the files, commited in that timeframe (deepgram.com)
   - [x] Link uploaded OneDrive audiofiles to git repo
   - [x] Transcribe
   - [x] Link to headers that have been commited to in the timeframe
- [ ] automatically create Anki flash cards -> Anki cloud integration? (using gemini)
   - [ ] Anki card creation
   - [ ] UI to check the knowledge coverage of those generated cards -> ignore/include metadata in the .md files
- [ ] automatically create a TOC in the main readme
- [ ] automatically tag the notes to make them easier seachable
- [ ] https://www.quivr.com/ integration (but has to use gemini)

## How?
### OneDrive file transcription pipeline
1. Check all files in the folder defined as `ONEDRIVE_SOURCE_DIR` in your OneDrive (with the extension from `PERMITTED_FILE_TYPES`)
2. Create a link file in your github repo in the branch `AUDIO_GIT_BRANCH` under `AUDIO_TARGET_DIR` (=> this for the audio files to be still stored in your OneDrive)
3. Use the link files to get a filelink of the OneDrive api (this allows us to save bandwidth)
4. Transcript the file using [DeepGram](https://deepgram.com/)
5. Store the transcript in the branch `TRANSCRIPTION_TARGET_PATH` in `TRANSCRIPTION_GIT_BRANCH` of your git repo
6. Search for files changed before the creation of the audiofile `TRANSCRIPTION_TIME_WINDOW`
7. Add the [Shodo-Notes](https://github.com/codecrafter404/shodo) header to it, in order to link it to the lessons

## Setup
0. Install rust, clone the repo & build the project `cargo build` or run it `cargo run -- --help`
1. Setup a simple http server which returnes a valid api key for usage with the OneDrive api
   - should return a 200 with the following structure: (this is due to microsoft having an awful api key management system :( )
   ```json
   {
      "token": "<your-token-here>",
      "scope": "should contain Files.Read"
   }
   ```
2. Get your [DeepGram API KEY](https://deepgram.com/)
3. Fill out the [.env.example](https://github.com/github/codecrafter404/korosensei/blob/main/.env.example) (and rename to `.env`) (documented / see [pipeline docs](#onedrive-file-transcription-pipeline))
4. Run the program (Get help with `--help` in order to activate/deactivate different steps of the pipeline)
