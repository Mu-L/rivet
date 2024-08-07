/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { JoinLobby as matchmaker_common$$joinLobby } from "../../common/types/JoinLobby";
import { JoinPort as matchmaker_common$$joinPort } from "../../common/types/JoinPort";
import { JoinPlayer as matchmaker_common$$joinPlayer } from "../../common/types/JoinPlayer";
import { matchmaker } from "../../../../index";

export const CreateLobbyResponse: core.serialization.ObjectSchema<
    serializers.matchmaker.CreateLobbyResponse.Raw,
    Rivet.matchmaker.CreateLobbyResponse
> = core.serialization.object({
    lobby: matchmaker_common$$joinLobby,
    ports: core.serialization.record(core.serialization.string(), matchmaker_common$$joinPort),
    player: matchmaker_common$$joinPlayer,
});

export declare namespace CreateLobbyResponse {
    interface Raw {
        lobby: matchmaker.JoinLobby.Raw;
        ports: Record<string, matchmaker.JoinPort.Raw>;
        player: matchmaker.JoinPlayer.Raw;
    }
}
