/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const Summary: core.serialization.ObjectSchema<serializers.cloud.version.Summary.Raw, Rivet.cloud.version.Summary>;
export declare namespace Summary {
    interface Raw {
        version_id: string;
        create_ts: serializers.Timestamp.Raw;
        display_name: serializers.DisplayName.Raw;
    }
}
