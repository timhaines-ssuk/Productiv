# Productiv Feature Status

Updated: 2026-03-09

Status legend:
- `Implemented`: working in the current local build
- `Partial`: scaffolded or locally modeled, but external integration or deeper behavior is still missing
- `Not started`: discussed, but not yet built

## Current Product Shape

| Feature | Status | Notes |
| --- | --- | --- |
| Tray-first desktop app shell | `Implemented` | Rust `eframe`/`egui` app with a system tray icon and a hidden root host window |
| Right-side widget UI | `Implemented` | Tray click opens a 500px-wide floating widget anchored to the right side of the screen |
| Minimalist card layout | `Implemented` | Busy multi-panel dashboard replaced with a compact card-based widget |
| Local SQLite storage in `LocalAppData` | `Implemented` | Database is stored at `%LocalAppData%\\Productiv\\productiv.sqlite3` |

## Planning And Time Management

| Feature | Status | Notes |
| --- | --- | --- |
| Full day itinerary view | `Implemented` | Vertical time-slot itinerary shown inside the widget |
| Draft tasks locally | `Implemented` | Tasks can be added directly from the widget |
| Drag tasks into a time slot | `Implemented` | Backlog/planned tasks can be dragged into itinerary slots |
| Drag existing planned blocks to reschedule | `Implemented` | Planned task blocks can be dragged onto a new time slot |
| Remove planned blocks | `Implemented` | Existing planned task blocks can be cleared from the itinerary |
| Default block duration | `Implemented` | Scheduling uses a configurable 30-180 minute default block size |
| Completed task effort rollup | `Implemented` | Closing a task rolls up tracked time first, then planned time if no tracked time exists |
| Offer to write hours back to work item | `Implemented` | Completion prompt can queue hours for remote writeback |
| Actual Azure DevOps hour writeback call | `Not started` | Only local queueing exists right now |

## Activity Tracking

| Feature | Status | Notes |
| --- | --- | --- |
| Foreground window polling | `Implemented` | Windows foreground app/window sampling runs in the background |
| Idle detection | `Implemented` | Idle periods are detected using Windows input state |
| Local activity timeline | `Implemented` | Activity segments are stored in SQLite and shown in the widget |
| Manual active task linking | `Implemented` | Foreground activity is linked to the manually active task when one is set |
| Low-overhead runtime loop | `Implemented` | Polling is set to 2 seconds by default and designed to stay lightweight |
| Automatic classification engine | `Partial` | Data model and intent exist, but only manual linking is active today |
| UI Automation inspection | `Not started` | No accessibility tree scraping yet |
| Browser URL / Chromium context detection | `Not started` | No Chrome DevTools Protocol integration yet |
| Audio session detection | `Not started` | No audio session inspection yet |
| File / repo / git context extraction | `Not started` | No editor-specific parsing or git inspection yet |

## Calendar And Work Item Integrations

| Feature | Status | Notes |
| --- | --- | --- |
| Outlook calendar model | `Partial` | Meetings are modeled and rendered in the itinerary |
| Outlook COM sync | `Not started` | Current build only seeds local placeholder meetings |
| Azure DevOps work item model | `Partial` | Tasks support external IDs and queued writeback metadata |
| Azure DevOps PAT configuration | `Implemented` | PAT, org URL, and project are editable in the Preferences modal |
| Azure DevOps task loading | `Not started` | No live REST sync yet |
| Azure DevOps field editing | `Not started` | No live REST update actions yet |
| Open work item in browser | `Not started` | Not wired yet |

## Configuration And Operational Details

| Feature | Status | Notes |
| --- | --- | --- |
| Preferences modal | `Implemented` | Widget contains a dedicated Preferences modal |
| Store PAT and runtime settings locally | `Implemented` | Settings are persisted in SQLite |
| Toggle Outlook / Azure DevOps integration flags | `Implemented` | Flags are stored and surfaced in the UI |
| Configure poll interval and idle threshold | `Implemented` | Runtime settings are editable and persisted |
| Secret storage in Windows Credential Manager | `Not started` | PAT is still stored in SQLite for now |
| Tray open / hide behavior | `Implemented` | Left-click tray toggles the widget open and closed |
| Full minimize-to-tray lifecycle | `Partial` | The widget is tray-driven, but deeper window lifecycle polish is still pending |

## Notes On Scope

- The current build is a strong local-first shell with planning, local persistence, and background tracking.
- The largest missing work is external integration: Outlook COM sync, Azure DevOps task sync, and Azure DevOps writeback execution.
- The current UI has been intentionally simplified to fit the tray-widget usage model rather than a full desktop dashboard.
