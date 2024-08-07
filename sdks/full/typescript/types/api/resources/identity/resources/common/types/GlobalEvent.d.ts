/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as Rivet from "../../../../../index";
/**
 * An event relevant to the current identity.
 */
export interface GlobalEvent {
    ts: Rivet.Timestamp;
    kind: Rivet.identity.GlobalEventKind;
    notification?: Rivet.identity.GlobalEventNotification;
}
