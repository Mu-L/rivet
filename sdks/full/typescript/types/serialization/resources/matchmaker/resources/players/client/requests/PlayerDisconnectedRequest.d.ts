/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../index";
import * as Rivet from "../../../../../../../api/index";
import * as core from "../../../../../../../core";
export declare const PlayerDisconnectedRequest: core.serialization.Schema<serializers.matchmaker.PlayerDisconnectedRequest.Raw, Rivet.matchmaker.PlayerDisconnectedRequest>;
export declare namespace PlayerDisconnectedRequest {
    interface Raw {
        player_token: string;
    }
}
