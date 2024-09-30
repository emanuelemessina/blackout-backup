# Blackout

_PoliTo - System & Devices Programming 2024 - Final Project (Group 8)_ 

[Presentation](doc/group8.pdf)

## Usage

Currently the application runs on Windows only (because of Echo and Tray), but the code is not too dependent on winapi
and it souldn't be hard to port it to other OS.
\
\
The chosen mouse trigger is a path around the border of the primary display: top left 🠒 bottom left 🠒 bottom right 🠒 top right.
\
To complete the path the mouse must never be further than 1/6 of the screen width from the borders of the screen,
otherwise the path must start over.
\
To trigger the backup the path must be completed twice (one for arming, one for triggering).
\
Arm, trigger and cancel (if you fall outside the path) events provide sound feedback.
\
\
The backup sources are specified in `sources.txt` as [glob](https://docs.rs/glob/latest/glob/#) paths.
\
\
While the backup is in progress it will play a heartbeat sound.
\
If an error occurs, an error sound is played.
\
If the backup completes succesfully a success sound is played.
\
Either way the heartbeat stops.
\
\
During the backup a console is displayed showing the backup status and progress, that closes after the backup
terminates.
\
General logs are available in `blackout.log`.
\
\
The backup is saved in the first found removable drive that has enough space to store the specified sources, under
`BlackoutBackup\<timestamp>`, keeping the original directory structure intact.
\
The backup specific log, containing the actual written files, sizes and execution time, is written at
`BlackoutBackup\<timestamp>.log`, next to the homonymous folder.
\
\
If the backup if successful the mouse path detection is rearmed.
\
If there are errors during the file copy it will try to complete the remaining files.
\
If there are fatal errors during backup, or if the backup thread panics, the mouse thread resumes probing and the path
detection is rearmed anyway.
\
Errors in other threads are fatal and cause the application to quit.

## Build from source

Build: `cargo build`
\
Run the postbuild script: `./post-build.ps1`
\
The build will be under `deploy/<target>`

## Install

Install: `.\install.ps1`
\
Uninstall: `.\uninstall.ps1`

The progam is automatically scheduled to start at user logon.

## Implementation details

### General

Each job is assigned to a specific thread.
\
Min CPU usage is achieved by minimizing the active waiting time (bus contention) of the threads.
\
Audio is played async by spawning tokio tasks.
\
A global application state coordinates the thread execution.
\
The installer registers the application for staring at user logon via a Windows Scheduled Task.
\
The application uses file locking to ensure a single instance running at all times.
\
Resources (images, audio) are embedded into the binary at compile time since they are small.

### Logger

The logger has its own thread that listens to a blocking channel of logs.
\
Each time a threads wants to log a line, it sends a message to the logger and the message goes into the channel queue
for the logger to receive and write to a file.
\
This way file writing is non blocking but still in order for the calling thread, and internally the channel manages the
wake/sleep cycle of
the logger thread.
\
The logger itself is a global object available to any thread, and it has no synch guards except the internal channel,
since writing logs is not critical.
\
The logger can work on multiple files at once since it stores a hashmap of open log files.

### Tray

The tray events triggering is managed internally by the operating system.
\
Events are sent to the tray thread via a blocking channel, thus the tray thread is awoken only when there are events to
process.
\
The tray thread is the only one allowed to quit the application gracefully by user command (the other threads cause the
application to quit because of fatal errors).
\
The tray thread is sync, so it uses a crossbeam select to listen either to a state change or to a tray action.

### Main thread

The main thread, apart from setting up the application and spawning the tray and mouse threads, maintains the general
state machine of the application and logs the cpu usage for the process.
\
Since it is marked as a tokio main, it uses a tokio select to listen to either a state change of a cpu log interval.

### Mouse listener

Since the mouse position is available at all times without events, the mouse thread is a low level loop that probes the
mouse position at time intervals via thread::sleep.
\
Once the backup is triggered it launches the Backup thread in and immediately joins to avoid a double backup triggering.
\
This way it also rearms the path detection once a backup finishes (either because of success or of error).

### Backup thread

The backup thread launches the Echo manager thread and sends messages to it to display ui messages.
\
It parses the sources file, finds a suitable removable drive (with enough free space to store the sources) and starts
the file copying.
\
It tries to go on even if a file fails, in the end it reports the actual files that were written succesfully.
\
The file copy is sequential, blocking and single threaded, as it doesn't make sense to issue multiple transfers at once
for a device that has a single bus and a single controller.

### Echo

Echo is a separate binary that endlessly prints wathever is sent to its stdin.
\
The echo thread spawns the echo process with piped stdio, and forwards the lines to be printed from the backup thread to
the echo process.
\
It also checks whether the process is active, and in case it's not, it respawns the process and prints the remaining
lines.
\
Echo can only be killed permanently from the backup thread by dropping the channel Sender when the backup thread does
not want to show text to the user anymore.
\
The reason it's a separate process is that it's easy to make it spawn in its own console window and to monitor whether
it was closed externally, because closing the window kills the process.
\
Thus is the window is closed the process can just restart in the background without the backup thread worrying about it.
\
It appears impossible to (at least on Windows) customize the console window to remove the close button (preventing the
user from closing the window) or doing other style changes.
\
Anything else than this would require setting up a custom ui from scratch either via the os gui library or via rendering
libraries, which is overkill for this application.
\
For these reasons the Echo approach seems the most suitable compromise to show the user the current status of the
backup. 


