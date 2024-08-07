/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { cloud } from "../../index";
export declare const BootstrapCaptcha: core.serialization.ObjectSchema<serializers.cloud.BootstrapCaptcha.Raw, Rivet.cloud.BootstrapCaptcha>;
export declare namespace BootstrapCaptcha {
    interface Raw {
        turnstile?: cloud.BootstrapCaptchaTurnstile.Raw | null;
    }
}
