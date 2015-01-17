/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::task;
use std::comm::Sender;
use std::task::TaskBuilder;
use std::thunk::Thunk;
// use rtinstrument;
use task_state;

pub fn spawn_named<S: IntoCow<'static, String, str>>(name: S, f: Thunk) {
    let builder = task::TaskBuilder::new().named(name);
    builder.spawn(move || {
        // rtinstrument::instrument(f);
        f();
    });
}

/// Arrange to send a particular message to a channel if the task fails.
pub fn spawn_named_with_send_on_failure<T: Send>(name: &'static str,
                                                 state: task_state::TaskState,
                                                 f: Thunk,
                                                 msg: T,
                                                 dest: Sender<T>) {
    let future_result = TaskBuilder::new().named(name).try_future(move || {
        task_state::initialize(state);
        // FIXME: Find replacement for this post-runtime removal
        // rtinstrument::instrument(f);
        f();
    });

    let watched_name = name.into_string();
    let watcher_name = format!("{}Watcher", watched_name);
    TaskBuilder::new().named(watcher_name).spawn(move || {
        //rtinstrument::instrument(move || {
            match future_result.into_inner() {
                Ok(()) => (),
                Err(..) => {
                    debug!("{} failed, notifying constellation", name);
                    dest.send(msg);
                }
            }
        //});
    });
}
