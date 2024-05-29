/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../../..";
import * as Rivet from "../../../../../../../../../../api";
import * as core from "../../../../../../../../../../core";
export declare const GameModeActions: core.serialization.ObjectSchema<serializers.cloud.version.matchmaker.GameModeActions.Raw, Rivet.cloud.version.matchmaker.GameModeActions>;
export declare namespace GameModeActions {
    interface Raw {
        find?: serializers.cloud.version.matchmaker.GameModeFindConfig.Raw | null;
        join?: serializers.cloud.version.matchmaker.GameModeJoinConfig.Raw | null;
        create?: serializers.cloud.version.matchmaker.GameModeCreateConfig.Raw | null;
    }
}