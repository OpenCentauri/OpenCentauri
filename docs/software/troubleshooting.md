# Troubleshooting

## Web UI shows no files, no history and no camera

Seen on stock firmware 1.4.49, but not specific to it.

Symptoms:

- The web UI shows "There is no file for now", the print history is empty and the camera's Play button does nothing. Slicer device tabs can show the same.
- The camera itself works: `http://<printer-ip>:3031/video` shows the stream.
- The browser console repeats `WebSocket connection to 'ws://<printer-ip>:3030/websocket' failed: Could not decode a text frame as UTF-8.`

### Cause

The printer keeps its own list of the files in `/local`, stored in `/user-resource/file_info/file_info.txt`. Sometimes the firmware writes an entry with invalid bytes in its name. The file list response ([Cmd 258](api.md#retrieve-file-list-cmd-258)) is then not valid UTF-8, and browsers are required to close a websocket that receives such a text frame. The web UI reconnects, asks for the file list again, and loops. Commands sent in between, such as starting the video, go to a closed socket.

In one case the broken entry was a copy of the factory sample "PTFE Tube Tool" with the first 8 bytes of its name overwritten, and with the same size and date as the real file:

```
/local/\x9ffF\xfc'\x02Y@PTFE Tube Tool_Elegoo PLA _0.2_21m9s.gcode
/local/ECC_0.4_PTFE Tube Tool_Elegoo PLA _0.2_21m9s.gcode
```

No file with the broken name existed on disk. Port 80 serves a plain directory listing of the real files at `http://<printer-ip>/local/`, which is handy for comparing.

These do not remove the entry:

- rebooting the printer;
- deleting ([Cmd 259](api.md#batch-delete-files-cmd-259)) or renaming (Cmd 257) the broken entry. The printer looks for the file on disk, does not find it and keeps the entry. Cmd 260 for that name returns `Ack: 2`, while Cmd 259 still answers `Ack: 0`;
- deleting the real file that the broken entry copies.

### Fix on patched firmware

OpenCentauri patched firmware runs `utf8-fix` on `file_info.txt` at every boot, replacing invalid bytes with `_`. A reboot is enough.

### Fix on stock firmware (no root needed)

Make the broken entry point to a real file, then delete that file the normal way:

1. Upload a small placeholder file whose name is exactly the broken name, as raw bytes, through `POST http://<printer-ip>/uploadFile/upload`. The printer then finds the file under that name.
2. Wait until Cmd 260 for that name answers `Ack: 0`, which takes a few seconds. A delete sent right after the upload is acknowledged but ignored.
3. Delete it with Cmd 259, sending the name exactly as the printer reported it.

The printer drops the entry, and the file list is valid UTF-8 again.

[fix_file_list_utf8.py](../assets/fix_file_list_utf8.py) does these steps for every broken entry. It needs only Python 3, without extra packages. The printer has to be idle, because uploads are refused while printing.

```
python3 fix_file_list_utf8.py <printer-ip> --dry-run   # only list broken entries
python3 fix_file_list_utf8.py <printer-ip>
```

A factory reset is also reported to clear the problem ([OpenCentauri#47](https://github.com/OpenCentauri/OpenCentauri/issues/47)), but it resets the printer's settings as well.
