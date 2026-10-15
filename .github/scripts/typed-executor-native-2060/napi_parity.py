"""Untimed CJS/ESM prepared API and full CLI parity for every typed entry."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

from common import require, write
from pair import HERE, ROOT, SPEC


def canonical(data,fixture):
    return json.loads(data.decode().replace(str(fixture),'<fixture>'))


def sha(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def ordered_entries(fixture_key,expectations):
    # The imported-utility collection uses physical source order. JSON key sorting
    # changes that order, and the full CJS/ESM report hash includes report IDs.
    if fixture_key=='imported-utility-order':
        entries=['early.mts','late.mts','late-shapes.mts','shadows.mts']
    else:
        entries=list(expectations)
    require(len(entries)==len(expectations) and set(entries)==set(expectations),
            'native_entry_catalog_changed',fixture=fixture_key)
    return entries


def run(root):
    private=root/'private/napi-parity';private.mkdir(parents=True,exist_ok=True)
    expected=SPEC['semantic_policy']['report_hashes']['napi']
    results={}
    cases=(('typed-projections','effects','candidate_api_effect_expectations',17,18,'candidate_effect_count'),
           ('imported-utility-order','utility','custom_utility_api_effect_expectations',4,5,'custom_utility_effect_count'))
    require(set(expected)=={case[0] for case in cases},'native_fixture_hash_coverage_incomplete')
    for fixture_key,prefix,expectation_key,entry_count,report_count,effect_total_key in cases:
        fixture=root/'inputs'/SPEC['inputs'][fixture_key]['path']
        expectations=SPEC['semantic_policy'][expectation_key]
        entries=ordered_entries(fixture_key,expectations)
        require(len(entries)==entry_count,'native_case_count_changed',fixture=fixture_key)
        results[fixture_key]={}
        for arm in ('baseline','candidate'):
            addon=root/arm/'transfer/addon.node'
            env=dict(os.environ,NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon))
            proc=subprocess.run(['node',str(HERE/'napi-parity.mjs'),str(ROOT/'packages/no-mistakes'),
                                 str(fixture),json.dumps(entries),prefix],
                                env=env,capture_output=True,timeout=SPEC['budgets_minutes']['napi_parity']*60)
            (private/f'{fixture_key}-{arm}.stdout').write_bytes(proc.stdout)
            (private/f'{fixture_key}-{arm}.stderr').write_bytes(proc.stderr)
            require(proc.returncode==0 and not proc.stderr,'native_cjs_esm_execution_failed',arm=arm,fixture=fixture_key)
            report=canonical(proc.stdout,fixture)
            require(len(report['reports'])==report_count,'native_report_count_changed',arm=arm,fixture=fixture_key)
            cli_hashes={}
            for index,entry in enumerate(entries):
                result=report['reports'][index]
                require(result['id']==f'{prefix}-{index}','native_report_order_changed',arm=arm,fixture=fixture_key)
                command=[str(root/arm/'transfer/cli'),'effects','postgres','--entry',entry,'--root',str(fixture),'--format','json']
                cli=subprocess.run(command,capture_output=True,timeout=60)
                (private/f'{fixture_key}-{arm}-{index}.stdout').write_bytes(cli.stdout)
                (private/f'{fixture_key}-{arm}-{index}.stderr').write_bytes(cli.stderr)
                require(cli.returncode==0 and not cli.stderr,'native_effects_cli_failed',arm=arm,entry=entry,fixture=fixture_key)
                cli_report=canonical(cli.stdout,fixture)
                require(result['result']==cli_report,'native_effects_cli_report_changed',arm=arm,entry=entry,fixture=fixture_key)
                cli_hashes[entry]=sha(cli_report)
                if arm=='candidate':
                    lines=[site['line'] for site in cli_report['callSites']]
                    require(lines==expectations[entry],'candidate_effect_lines_changed',entry=entry,actual=lines,fixture=fixture_key)
            require(report['reports'][-1]['id']=='check','native_check_order_changed',arm=arm,fixture=fixture_key)
            check=subprocess.run([str(root/arm/'transfer/cli'),'check','--root',str(fixture),'--format','json'],capture_output=True,timeout=60)
            (private/f'{fixture_key}-{arm}-check.stdout').write_bytes(check.stdout)
            (private/f'{fixture_key}-{arm}-check.stderr').write_bytes(check.stderr)
            require(check.returncode in (0,1) and not check.stderr,'native_check_cli_failed',arm=arm,fixture=fixture_key)
            check_report=canonical(check.stdout,fixture)
            require(report['reports'][-1]['result']==check_report,'native_check_cli_report_changed',arm=arm,fixture=fixture_key)
            require(not check_report['warnings'],'native_check_warnings',arm=arm,fixture=fixture_key)
            digest=sha(report)
            require(digest==expected[fixture_key][arm],'native_full_report_hash_changed',arm=arm,actual=digest,fixture=fixture_key)
            effects_count=sum(len(report['reports'][i]['result']['callSites']) for i in range(entry_count))
            if arm=='candidate':
                require(effects_count==SPEC['semantic_policy'][effect_total_key],
                        'candidate_effect_total_changed',actual=effects_count,fixture=fixture_key)
            results[fixture_key][arm]={'full_report_sha256':digest,'entries':cli_hashes,
                                       'check_sha256':sha(check_report),'effects_count':effects_count}
    write(root/'public/napi-parity.json',results)
    return results
