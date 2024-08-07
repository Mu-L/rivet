/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../index";
import * as Rivet from "../../../../../../../api/index";
import * as core from "../../../../../../../core";
import { matchmaker, captcha } from "../../../../../index";
export declare const CreateLobbyRequest: core.serialization.Schema<serializers.matchmaker.CreateLobbyRequest.Raw, Rivet.matchmaker.CreateLobbyRequest>;
export declare namespace CreateLobbyRequest {
    interface Raw {
        game_mode: string;
        region?: string | null;
        publicity?: matchmaker.CustomLobbyPublicity.Raw | null;
        tags?: Record<string, string> | null;
        max_players?: number | null;
        lobby_config?: unknown | null;
        captcha?: captcha.Config.Raw | null;
        verification_data?: unknown | null;
    }
}
