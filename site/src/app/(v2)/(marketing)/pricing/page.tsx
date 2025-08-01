import type { Metadata } from "next";
import PricingPageClient from "./PricingPageClient";

export const metadata: Metadata = {
	title: "Pricing - Rivet",
	description:
		"See Rivet's transparent pricing for serverless compute—billed by the millisecond for functions, actors, and containers. No hidden fees.",
	alternates: {
		canonical: "https://www.rivet.gg/pricing/",
	},
};

export default function Page() {
	return <PricingPageClient />;
}
