# tasks
- [x] preview navigation
- [ ] add a way to open the selected file in the default editor
- [x] maybe filter out image types etc. for now
- [x] return selected entry on exit
- [x] piping output to another command
- [x] piping custom entries from stdin (e.g. `ls | tv`, what bout choosing previewers in that case? Some AUTO mode?)

## bugs
- [x] sanitize input (tabs, \0, etc) (see https://github.com/autobib/nucleo-picker/blob/d51dec9efd523e88842c6eda87a19c0a492f4f36/src/lib.rs#L212-L227)

## improvements
- [x] async finder initialization
- [x] async finder search
- [x] use nucleo for env
- [ ] better keymaps
- [ ] mutualize placeholder previews in cache (really not a priority)
- [ ] better abstractions for channels / separation / isolation so that others can contribute new ones easily
- [ ] channel selection in the UI (separate menu or top panel or something)
- [x] only render highlighted lines that are visible
- [x] only ever read a portion of the file for the temp preview
- [ ] make layout an attribute of the channel?
- [ ] I feel like the finder abstraction is a superfluous layer, maybe just use the channel directly?

## feature ideas
- [ ] some sort of iterative fuzzy file explorer (preview contents of folders on the right, enter to go in etc.) maybe
  with mixed previews of files and folders
- [x] environment variables
- [ ] aliases
- [ ] shell history
- [x] text
- [ ] text in documents (pdfs, archives, ...) (rga, adapters) https://github.com/jrmuizel/pdf-extract
- [x] fd
- [ ] recent directories
- [ ] git (commits, branches, status, diff, ...)
- [ ] makefile commands
- [ ] remote files (s3, ...)
- [ ] custom actions as part of a channel (mappable)

