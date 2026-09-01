import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import Shell from 'gi://Shell';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

// This extension does not collect, store, or send anything itself. Its
// only job is to expose the currently focused application over D-Bus,
// because Wayland does not let a normal background process (the
// scremetime daemon) query focused window information for itself. Only
// GNOME Shell is allowed to know this, so the extension runs inside the
// shell and reports it on the shell's own D-Bus name.
const IFACE_XML = `
<node>
  <interface name="org.gnome.Shell.Extensions.Scremetime">
    <method name="GetFocusedApp">
      <arg type="s" direction="out" name="app_id"/>
    </method>
    <signal name="FocusedAppChanged">
      <arg type="s" name="app_id"/>
    </signal>
  </interface>
</node>`;

export default class ScremetimeExtension extends Extension {
    enable() {
        this._tracker = Shell.WindowTracker.get_default();

        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(IFACE_XML, this);
        this._dbusImpl.export(Gio.DBus.session, '/org/gnome/Shell/Extensions/Scremetime');

        this._focusChangedId = this._tracker.connect('notify::focus-app', () => {
            this._emitFocusedApp();
        });
    }

    disable() {
        if (this._focusChangedId) {
            this._tracker.disconnect(this._focusChangedId);
            this._focusChangedId = null;
        }
        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }
        this._tracker = null;
    }

    // Called by GetFocusedApp over D-Bus. Lets the daemon ask for the
    // current state directly instead of only waiting for the next signal,
    // which matters right after the daemon starts up.
    GetFocusedApp() {
        const app = this._tracker.focus_app;
        return app ? app.get_id() : '';
    }

    _emitFocusedApp() {
        const app = this._tracker.focus_app;
        const appId = app ? app.get_id() : '';
        this._dbusImpl.emit_signal('FocusedAppChanged', new GLib.Variant('(s)', [appId]));
    }
}
