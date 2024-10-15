# tasks
- [x] preview navigation
- [ ] add a way to open the selected file in the default editor (or maybe that should be achieved using pipes?)
- [x] maybe filter out image types etc. for now
- [x] return selected entry on exit
- [x] piping output to another command
- [x] piping custom entries from stdin (e.g. `ls | tv`, what bout choosing previewers in that case? Some AUTO mode?)

## improvements
- [x] async finder initialization
- [x] async finder search
- [x] use nucleo for env
- [ ] better keymaps
- [ ] mutualize placeholder previews in cache (really not a priority)
- [x] better abstractions for channels / separation / isolation so that others can contribute new ones easily
- [ ] channel selection in the UI (separate menu or top panel or something)
- [x] only render highlighted lines that are visible
- [x] only ever read a portion of the file for the temp preview
- [ ] make layout an attribute of the channel?
- [x] I feel like the finder abstraction is a superfluous layer, maybe just use the channel directly?
- [x] support for images is implemented but do we really want that in the core? it's quite heavy
- [ ] use an icon for the prompt
- [ ] profile using dyn Traits instead of an enum for channels (might degrade performance by storing on the heap)

## feature ideas
- [ ] some sort of iterative fuzzy file explorer (preview contents of folders on the right, enter to go in etc.) maybe
  with mixed previews of files and folders
- [x] environment variables
- [x] aliases
- [ ] shell history
- [x] text
- [ ] text in documents (pdfs, archives, ...) (rga, adapters) https://github.com/jrmuizel/pdf-extract
- [x] fd
- [ ] recent directories
- [ ] git (commits, branches, status, diff, ...)
- [ ] makefile commands
- [ ] remote files (s3, ...)
- [ ] custom actions as part of a channel (mappable)
- [ ] from one set of entries to another? (fuzzy-refine)

