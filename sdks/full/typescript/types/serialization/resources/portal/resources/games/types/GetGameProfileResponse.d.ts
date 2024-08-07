/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { game, common } from "../../../../index";
export declare const GetGameProfileResponse: core.serialization.ObjectSchema<serializers.portal.GetGameProfileResponse.Raw, Rivet.portal.GetGameProfileResponse>;
export declare namespace GetGameProfileResponse {
    interface Raw {
        game: game.Profile.Raw;
        watch: common.WatchResponse.Raw;
    }
}
