/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../../../..";
import * as Rivet from "../../../../../../../../../../api";
import * as core from "../../../../../../../../../../core";
export declare const GameModeFindConfig: core.serialization.ObjectSchema<serializers.cloud.version.matchmaker.GameModeFindConfig.Raw, Rivet.cloud.version.matchmaker.GameModeFindConfig>;
export declare namespace GameModeFindConfig {
    interface Raw {
        enabled: boolean;
        identity_requirement?: serializers.cloud.version.matchmaker.GameModeIdentityRequirement.Raw | null;
        verification?: serializers.cloud.version.matchmaker.GameModeVerificationConfig.Raw | null;
    }
}
