<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="description" content="Refracted Terms of Service, Privacy Policy, and project disclaimer.">
    <meta name="color-scheme" content="dark">

    <title>Legal — {{ config('app.name', 'Refracted') }}</title>

    <link rel="icon" href="{{ asset('images/brand/refracted-icon.png') }}" type="image/png">
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=sora:400,500,600|space-grotesk:500,600,700&display=swap" rel="stylesheet" />
    @vite(['resources/css/app.css', 'resources/js/app.js'])
</head>
<body class="ref-canvas min-h-[100svh] font-sans text-grit-text antialiased">
    <div
        class="ref-wallpaper"
        style="--ref-wallpaper: url('{{ asset('images/brand/refracted-wallpaper.png') }}')"
        aria-hidden="true"
    ></div>

    <main class="mx-auto max-w-3xl px-6 py-16 sm:px-8 lg:py-20">
        <p class="font-display text-xs font-semibold uppercase tracking-[0.2em] text-signal">Legal</p>
        <h1 class="mt-3 font-display text-4xl font-semibold tracking-[-0.02em] text-grit-text">
            Terms &amp; Privacy
        </h1>
        <p class="mt-4 leading-relaxed text-grit-mist">
            Project disclaimer, Terms of Service, and Privacy Policy for Refracted.
        </p>

        <nav aria-label="On this page" class="mt-8 flex flex-wrap gap-3 border border-grit-line bg-grit-surface/60 p-4 text-xs uppercase tracking-[0.14em]">
            <a href="#disclaimer" class="text-grit-mist transition hover:text-signal">Disclaimer</a>
            <span class="text-grit-line">/</span>
            <a href="#terms" class="text-grit-mist transition hover:text-signal">Terms of Service</a>
            <span class="text-grit-line">/</span>
            <a href="#privacy" class="text-grit-mist transition hover:text-signal">Privacy Policy</a>
        </nav>

        <section id="disclaimer" class="mt-14 scroll-mt-24">
            <h2 class="font-display text-2xl font-semibold tracking-[-0.02em] text-grit-text">
                Project disclaimer
            </h2>
            <div class="mt-6 text-sm leading-relaxed text-grit-mist sm:text-base">
                <p>
                    Refracted is an independent community project for education, research, and preservation.
                    It is <strong class="text-grit-text">not affiliated with, endorsed by, sponsored by, or connected to</strong>
                    Electronic Arts Inc. (“EA”), DICE (EA Digital Illusions CE AB), or any other rights holder or associated entity.
                </p>

                <div class="mt-6 divide-y divide-grit-line border border-grit-line bg-grit-panel/80">
                    <div class="p-5 sm:p-6">
                        <h3 class="font-display text-sm font-semibold uppercase tracking-[0.16em] text-grit-text">Clean-room reimplementation</h3>
                        <p class="mt-3">
                            Refracted is a <strong class="text-grit-text">clean-room / white-room reimplementation</strong> of network service
                            behavior based on publicly observable client–server interaction, independently written documentation, and original engineering.
                            It does <strong class="text-grit-text">not</strong> include, embed, or redistribute
                            proprietary publisher source code, object code, SDKs, or other copyrighted software belonging to EA or any third party.
                            The project is not derived from publisher source trees or SDK codebases.
                        </p>
                    </div>

                    <div class="p-5 sm:p-6">
                        <h3 class="font-display text-sm font-semibold uppercase tracking-[0.16em] text-grit-text">No copyrighted game content</h3>
                        <p class="mt-3">
                            This project does <strong class="text-grit-text">not contain, distribute, host, or provide</strong> copyrighted game assets,
                            binaries, maps, models, audio, textures, DRM circumvention tools, or other proprietary game files.
                            You must supply any game software yourself through lawful means. Refracted only provides independently developed
                            service-layer software intended for research, documentation, and preservation scenarios.
                        </p>
                    </div>

                    <div class="p-5 sm:p-6">
                        <h3 class="font-display text-sm font-semibold uppercase tracking-[0.16em] text-grit-text">Descriptive &amp; compatibility reference only</h3>
                        <p class="mt-3">
                            References to game titles, engines, protocols, or brands are for identification, interoperability description,
                            and fair nominative / descriptive use only. They do not imply endorsement, sponsorship, partnership, or affiliation
                            with any rights holder. All trademarks remain the property of their respective owners, including without limitation
                            Battlefield, Command &amp; Conquer, Frostbite, and related marks of EA and its licensors.
                        </p>
                    </div>

                    <div class="p-5 sm:p-6">
                        <h3 class="font-display text-sm font-semibold uppercase tracking-[0.16em] text-grit-text">Non-commercial project</h3>
                        <p class="mt-3">
                            Refracted is free to use. It is not sold as a commercial product and access is not paywalled.
                            Optional voluntary donations may be accepted solely to help cover hosting, bandwidth, and related infrastructure.
                            Donations do not purchase features, status, influence, or rights in any publisher IP, and do not create a commercial
                            relationship with EA or any rights holder.
                        </p>
                    </div>

                    <div class="p-5 sm:p-6">
                        <h3 class="font-display text-sm font-semibold uppercase tracking-[0.16em] text-grit-text">Your responsibilities</h3>
                        <p class="mt-3">
                            You are solely responsible for using Refracted only in ways that comply with applicable law and with the terms
                            that apply to software and services you use (including game end-user agreements). The maintainers do not encourage
                            or support piracy, cheating, unauthorized access to live commercial infrastructure, or circumvention of technical
                            protection measures for live service titles.
                        </p>
                    </div>
                </div>
            </div>
        </section>

        <section id="terms" class="mt-16 scroll-mt-24 border-t border-grit-line/80 pt-14">
            <h2 class="font-display text-2xl font-semibold tracking-[-0.02em] text-grit-text">
                Terms of Service
            </h2>
            <p class="mt-2 text-xs uppercase tracking-[0.14em] text-grit-mist">
                Effective date: {{ now()->format('d/m/Y') }}
            </p>

            <div class="mt-8 space-y-8 text-sm leading-relaxed text-grit-mist sm:text-base">
                <p>
                    Welcome to Refracted (“we,” “our,” or “us”). By accessing or using Refracted’s websites, software,
                    or related services (collectively, the “Service”), you agree to these Terms of Service (“Terms”).
                    If you do not agree, do not use the Service.
                </p>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">1. Rules of conduct</h3>
                    <ul class="mt-4 list-disc space-y-2 ps-5">
                        <li><strong class="text-grit-text">Lawful use only:</strong> Use Refracted only in ways that comply with applicable law and the terms that apply to software you use.</li>
                        <li><strong class="text-grit-text">No piracy or content redistribution:</strong> Do not use Refracted to obtain, share, host, or distribute proprietary game files, assets, or copyrighted publisher software.</li>
                        <li><strong class="text-grit-text">No abuse of live services:</strong> Do not use Refracted to attack, disrupt, or gain unauthorized access to publisher or third-party live infrastructure.</li>
                        <li><strong class="text-grit-text">No cheating support:</strong> Do not use Refracted to develop or distribute cheats, bots, or tools intended to unfairly disrupt others in online play.</li>
                        <li><strong class="text-grit-text">Respect the community:</strong> No harassment, hate speech, or harmful behavior toward other users or the project team.</li>
                    </ul>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">2. Intellectual property</h3>
                    <p class="mt-3">
                        Refracted’s independently authored software, documentation, and branding are owned by the project and its contributors,
                        subject to any open-source licenses published with the source.
                        Project branding may not be used or misrepresented without permission.
                    </p>
                    <p class="mt-4">
                        Refracted is <strong class="text-grit-text">not affiliated with or endorsed by Electronic Arts, DICE, or any other rights holder</strong>. Specifically:
                    </p>
                    <ul class="mt-4 list-disc space-y-2 ps-5">
                        <li>Any reference to third-party intellectual property is for non-commercial, descriptive, or compatibility purposes only.</li>
                        <li>Publisher trademarks, logos, and copyrighted materials remain the property of their respective owners.</li>
                        <li>Refracted does not distribute proprietary game binaries, assets, or copyrighted publisher code.</li>
                        <li>Refracted’s service layers are an original clean-room implementation and are not a copy of publisher source or proprietary codebases.</li>
                        <li>Users must adhere to applicable publisher policies when interacting with publisher products alongside Refracted.</li>
                    </ul>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">3. Limitation of liability</h3>
                    <p class="mt-3">
                        Refracted is provided “as is” without warranties of any kind, express or implied.
                        We are not responsible for service interruptions, data loss, damages, or legal consequences arising from your use of the Service.
                        Your use of Refracted is at your own risk.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">4. Donations &amp; support</h3>
                    <p class="mt-3">
                        Optional donations may help cover infrastructure costs. They are never required and do not purchase advantages,
                        exclusive access, or ownership interest. Third-party payment processors handle payment details under their own terms.
                        Refracted does not sell proprietary game content.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">5. Changes to the Terms</h3>
                    <p class="mt-3">
                        We may update these Terms from time to time.
                        Continued use of Refracted after updates constitutes acceptance of the revised Terms.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">6. Contact</h3>
                    <p class="mt-3">
                        Questions:
                        Discord
                        <span class="text-signal-soft">{{ config('services.contact.discord') }}</span>
                        or
                        <a href="{{ config('services.contact.telegram_url') }}" target="_blank" rel="noopener noreferrer" class="text-signal-soft underline decoration-signal/30 underline-offset-2">Telegram {{ '@' . config('services.contact.telegram') }}</a>.
                    </p>
                </div>
            </div>
        </section>

        <section id="privacy" class="mt-16 scroll-mt-24 border-t border-grit-line/80 pt-14">
            <h2 class="font-display text-2xl font-semibold tracking-[-0.02em] text-grit-text">
                Privacy Policy
            </h2>
            <p class="mt-2 text-xs uppercase tracking-[0.14em] text-grit-mist">
                Effective date: {{ now()->format('d/m/Y') }}
            </p>

            <div class="mt-8 space-y-8 text-sm leading-relaxed text-grit-mist sm:text-base">
                <p>
                    This Privacy Policy explains what data we may collect when you use the Refracted website and related online services.
                </p>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">1. Data we may collect</h3>
                    <ul class="mt-4 list-disc space-y-2 ps-5">
                        <li><strong class="text-grit-text">Account information:</strong> if you sign in to admin tools, Discord identity details needed for authentication.</li>
                        <li><strong class="text-grit-text">Technical data:</strong> IP address, browser metadata, and logs needed for security and reliability.</li>
                        <li><strong class="text-grit-text">Support communications:</strong> messages you send via Discord or Telegram.</li>
                        <li><strong class="text-grit-text">Donation-related data:</strong> limited confirmation details from payment providers if you choose to donate.</li>
                    </ul>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">2. How we use data</h3>
                    <ul class="mt-4 list-disc space-y-2 ps-5">
                        <li>Operate and improve the website and related services.</li>
                        <li>Authenticate administrators and prevent misuse.</li>
                        <li>Provide support and analyze aggregated usage to guide development.</li>
                    </ul>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">3. Sharing</h3>
                    <p class="mt-3">
                        We do not sell personal data. Data may be shared only as required by law or with processors needed to host the Service.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">4. Retention &amp; rights</h3>
                    <p class="mt-3">
                        We retain data only as long as needed to operate the Service or meet legal obligations.
                        You may request access, correction, or deletion by contacting
                        Discord
                        <span class="text-signal-soft">{{ config('services.contact.discord') }}</span>
                        or
                        <a href="{{ config('services.contact.telegram_url') }}" target="_blank" rel="noopener noreferrer" class="text-signal-soft underline decoration-signal/30 underline-offset-2">Telegram {{ '@' . config('services.contact.telegram') }}</a>.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">5. Children’s privacy</h3>
                    <p class="mt-3">
                        Refracted’s website is not intended for users under 13.
                        We do not knowingly collect data from children under 13.
                    </p>
                </div>

                <div>
                    <h3 class="font-display text-lg font-semibold tracking-[-0.01em] text-grit-text">6. Contact</h3>
                    <p class="mt-3">
                        Privacy questions:
                        Discord
                        <span class="text-signal-soft">{{ config('services.contact.discord') }}</span>
                        or
                        <a href="{{ config('services.contact.telegram_url') }}" target="_blank" rel="noopener noreferrer" class="text-signal-soft underline decoration-signal/30 underline-offset-2">Telegram {{ '@' . config('services.contact.telegram') }}</a>.
                    </p>
                </div>
            </div>
        </section>

        <p class="mt-14 border-t border-grit-line/80 pt-8 text-sm text-grit-mist">
            By using Refracted, you acknowledge that you have read and agree to both our Terms of Service and Privacy Policy.
        </p>

        <p class="mt-8 text-xs text-grit-mist/70">
            &copy; {{ date('Y') }} Refracted. Not affiliated with Electronic Arts Inc. or EA Digital Illusions CE AB.
            <a href="{{ url('/') }}" class="underline decoration-grit-line underline-offset-2 hover:text-grit-text">Home</a>
        </p>
    </main>
</body>
</html>


