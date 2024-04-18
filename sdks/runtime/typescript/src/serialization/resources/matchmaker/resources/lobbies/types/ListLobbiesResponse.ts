/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";

export const ListLobbiesResponse: core.serialization.ObjectSchema<
    serializers.matchmaker.ListLobbiesResponse.Raw,
    Rivet.matchmaker.ListLobbiesResponse
> = core.serialization.object({
    gameModes: core.serialization.property(
        "game_modes",
        core.serialization.list(
            core.serialization.lazyObject(async () => (await import("../../../../..")).matchmaker.GameModeInfo)
        )
    ),
    regions: core.serialization.list(
        core.serialization.lazyObject(async () => (await import("../../../../..")).matchmaker.RegionInfo)
    ),
    lobbies: core.serialization.list(
        core.serialization.lazyObject(async () => (await import("../../../../..")).matchmaker.LobbyInfo)
    ),
});

export declare namespace ListLobbiesResponse {
    interface Raw {
        game_modes: serializers.matchmaker.GameModeInfo.Raw[];
        regions: serializers.matchmaker.RegionInfo.Raw[];
        lobbies: serializers.matchmaker.LobbyInfo.Raw[];
    }
}