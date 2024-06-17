
CREATE TABLE IF NOT EXISTS users (
  user_id TEXT PRIMARY KEY,
  password VARCHAR(255) NOT NULL
);

REPLACE INTO users (user_id, password) VALUES ('admin', '______________');

CREATE TABLE IF NOT EXISTS chatrooms (
  chatroom_id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS chatroom_users (
  chatroom_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  PRIMARY KEY (chatroom_id, user_id),
  FOREIGN KEY (chatroom_id) REFERENCES chatrooms(chatroom_id),
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS messages (
  -- message_id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  chatroom_id TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at DATETIME NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(user_id),
  FOREIGN KEY (chatroom_id) REFERENCES chatrooms(chatroom_id)
);

CREATE TABLE IF NOT EXISTS timesheets (
  timesheet_id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  clocked_in BOOLEAN NOT NULL,
  current_id INTEGER, -- NULL if clocked out, otherwise the time_entry_id
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS time_entries (
  time_entry_id INTEGER PRIMARY KEY AUTOINCREMENT,
  timesheet_id INTEGER NOT NULL,
  start_time DATETIME,
  end_time DATETIME,
  start_note TEXT,
  end_note TEXT,
  FOREIGN KEY (timesheet_id) REFERENCES timesheets(timesheet_id)
);