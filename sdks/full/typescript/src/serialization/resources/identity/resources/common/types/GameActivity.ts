/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Handle as game_common$$handle } from "../../../../game/resources/common/types/Handle";
import { game } from "../../../../index";

export const GameActivity: core.serialization.ObjectSchema<
    serializers.identity.GameActivity.Raw,
    Rivet.identity.GameActivity
> = core.serialization.object({
    game: game_common$$handle,
    message: core.serialization.string(),
    publicMetadata: core.serialization.property("public_metadata", core.serialization.unknown().optional()),
    mutualMetadata: core.serialization.property("mutual_metadata", core.serialization.unknown().optional()),
});

export declare namespace GameActivity {
    interface Raw {
        game: game.Handle.Raw;
        message: string;
        public_metadata?: unknown | null;
        mutual_metadata?: unknown | null;
    }
}
