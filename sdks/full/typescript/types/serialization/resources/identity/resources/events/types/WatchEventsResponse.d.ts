/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const WatchEventsResponse: core.serialization.ObjectSchema<serializers.identity.WatchEventsResponse.Raw, Rivet.identity.WatchEventsResponse>;
export declare namespace WatchEventsResponse {
    interface Raw {
        events: serializers.identity.GlobalEvent.Raw[];
        watch: serializers.WatchResponse.Raw;
    }
}