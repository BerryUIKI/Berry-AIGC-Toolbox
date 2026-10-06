Additional native verification on 2026-10-05: after a rapid scrollbar drag triggered a second page append, Ctrl+A selected **800/800**, while the same folder still contained **1,276** indexed images. Before page append it selected 400/400.

This confirms that the current action's scope depends on the loaded prefix, rather than the full result population. The selection was cleared after inspection; no batch mutation or destructive operation was executed.
