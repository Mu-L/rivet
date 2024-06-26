/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const ValidateGameRequest: core.serialization.ObjectSchema<serializers.cloud.games.ValidateGameRequest.Raw, Rivet.cloud.games.ValidateGameRequest>;
export declare namespace ValidateGameRequest {
    interface Raw {
        display_name: string;
        name_id?: string | null;
    }
}
