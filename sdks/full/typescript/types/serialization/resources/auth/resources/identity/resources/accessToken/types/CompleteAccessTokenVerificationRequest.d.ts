/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const CompleteAccessTokenVerificationRequest: core.serialization.ObjectSchema<serializers.auth.identity.CompleteAccessTokenVerificationRequest.Raw, Rivet.auth.identity.CompleteAccessTokenVerificationRequest>;
export declare namespace CompleteAccessTokenVerificationRequest {
    interface Raw {
        access_token: serializers.Jwt.Raw;
    }
}
