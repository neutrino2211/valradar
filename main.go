package main

import (
	"fmt"
        "os"
	"os/signal"
	"regexp"
	"syscall"

	"github.com/alecthomas/kong"
	"github.com/fatih/color"
	"github.com/jamesits/goinvoke"
	"github.com/neutrino2211/go-result"
	"github.com/playwright-community/playwright-go"

	//#include<stdio.h>
	"C"
)

var browser playwright.Browser

type ValRadar struct {
	Site               string `required short:"s" name:"site" help:"The website to scan"`
	Depth              uint   `short:"d" name:"depth" help:"How deep to search" default:"1"`
	Concurrency        uint   `short:"c" name:"concurrency" help:"How many coroutines to use" default:"10"`
	Regex              string `required short:"p" name:"pattern" help:"The regex pattern to try matching"`
        UseHeadlessBrowser bool   `name:"use-headless-browser" help:"Use a headless chrome browser to fetch the webpages"`
}

func (v *ValRadar) Run(globals *ValRadar) error {
	re := result.SomePair(regexp.Compile(globals.Regex)).Expect("unable to compile the regex pattern " + globals.Regex)
	sm := NewSiteMap(globals.Site)
	ccr := NewCCR(int(globals.Concurrency))

	stateStorage := ""

	// Set up signal handling
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	// Ensure cleanup happens
	defer func() {
		<-sigChan
		print("Exiting...") // Not sure if I need the terminal input cleanup?
		os.Exit(0)
	}()
        
        if globals.UseHeadlessBrowser {
		playwright.Install(&playwright.RunOptions{
			Browsers: []string{"chromium"},
			Verbose:  false,
		})
		pw := result.SomePair(playwright.Run()).Expect("unable to run playwright")
		browser = result.SomePair(pw.Chromium.Launch()).Expect("unable to launch chromium")
		sm.fetcherFunc = func(s string) string {
			return fetchWithChrome(s, stateStorage)
		}
	}

	BuildSiteMap(ccr, sm, sm.url, 0, int(globals.Depth))

	ccr.wait()
	sm.spinner.Stop()

	found := 0

	for p, r := range sm.resources {
		matches := re.FindAllString(r.content, -1)
		for _, match := range matches {
			found += 1
			fmt.Println("🔎 Found " + color.HiGreenString(match) + " at the url " + color.GreenString(p))
		}
	}

	if found == 0 {
		fmt.Println(color.RedString("No matches found for " + globals.Regex))
	}

	os.Exit(0)

	return nil
}

func fetchWithChrome(url string, statePath string) string {
	userAgent := "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36"
	jsEnabled := true
	waitUntil := "load"
	waitUntilDomLoaded := "domcontentloaded"

	loadTimeout := float64(7000)
	resHtml := result.Try(func() string {
		page := result.SomePair(browser.NewPage(playwright.BrowserNewPageOptions{
			UserAgent:         &userAgent,
			JavaScriptEnabled: &jsEnabled,
		})).Expect("unable to create playwright page")

		if statePath != "" {
			result.SomePair(page.Context().StorageState(statePath)).Expect("unable to get storage state")
		}

		if _, err := page.Goto(url, playwright.PageGotoOptions{
			Timeout:   &loadTimeout,
			WaitUntil: (*playwright.WaitUntilState)(&waitUntil),
		}); err != nil {
			result.SomePair(page.Goto(url, playwright.PageGotoOptions{
				Timeout:   &loadTimeout,
				WaitUntil: (*playwright.WaitUntilState)(&waitUntilDomLoaded),
			})).Expect("unable to visit page " + url + " with playwright")
		}

		resHtml := result.SomePair(page.Locator(`html`).InnerHTML()).Expect("unable to get page inner html for " + url)
		page.Close()

		return resHtml
	})

	if resHtml.Error() != "" {
		println("\n ❌ " + resHtml.Error())
	}

	return resHtml.Or("")
}

type ValradarPlugin struct {
	VALRADAR_ARGS **C.char
}

func main() {
        // wip: loading plugins
	plugin := ValradarPlugin{}
	goinvoke.Unmarshal("plugins/test_c/c_plugin.dylib", &plugin)

	ctx := kong.Parse(
		&ValRadar{},
		kong.Name("valradar"),
		kong.Description("Search for patterns and strings over a website's footprint"),
		kong.UsageOnError(),
	)
	err := ctx.Run()
	if err != nil {
		panic(err)
	}
}
