/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common } from "../../../../index";
export declare const Handle: core.serialization.ObjectSchema<serializers.game.Handle.Raw, Rivet.game.Handle>;
export declare namespace Handle {
    interface Raw {
        game_id: string;
        name_id: common.Identifier.Raw;
        display_name: common.DisplayName.Raw;
        logo_url?: string | null;
        banner_url?: string | null;
    }
}
