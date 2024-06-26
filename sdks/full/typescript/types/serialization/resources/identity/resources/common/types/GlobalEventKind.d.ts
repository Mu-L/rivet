/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../..";
import * as Rivet from "../../../../../../api";
import * as core from "../../../../../../core";
export declare const GlobalEventKind: core.serialization.ObjectSchema<serializers.identity.GlobalEventKind.Raw, Rivet.identity.GlobalEventKind>;
export declare namespace GlobalEventKind {
    interface Raw {
        identity_update?: serializers.identity.GlobalEventIdentityUpdate.Raw | null;
        matchmaker_lobby_join?: serializers.identity.GlobalEventMatchmakerLobbyJoin.Raw | null;
    }
}
