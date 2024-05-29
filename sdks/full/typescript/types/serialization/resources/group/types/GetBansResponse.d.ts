/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../..";
import * as Rivet from "../../../../api";
import * as core from "../../../../core";
export declare const GetBansResponse: core.serialization.ObjectSchema<serializers.group.GetBansResponse.Raw, Rivet.group.GetBansResponse>;
export declare namespace GetBansResponse {
    interface Raw {
        banned_identities: serializers.group.BannedIdentity.Raw[];
        anchor?: string | null;
        watch: serializers.WatchResponse.Raw;
    }
}