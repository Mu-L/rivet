/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../../..";
import * as Rivet from "../../../../../../../../../../api";
import * as core from "../../../../../../../../../../core";
export declare const GameModeCreateConfig: core.serialization.ObjectSchema<serializers.cloud.version.matchmaker.GameModeCreateConfig.Raw, Rivet.cloud.version.matchmaker.GameModeCreateConfig>;
export declare namespace GameModeCreateConfig {
    interface Raw {
        enabled: boolean;
        identity_requirement?: serializers.cloud.version.matchmaker.GameModeIdentityRequirement.Raw | null;
        verification?: serializers.cloud.version.matchmaker.GameModeVerificationConfig.Raw | null;
        enable_public?: boolean | null;
        enable_private?: boolean | null;
        max_lobbies_per_identity?: number | null;
    }
}