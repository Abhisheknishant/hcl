#!/usr/sbin/dtrace -s
/* 
 * This script counts number of bytes read by executable,
 * prints it out every 5 seconds.
 */

#pragma D option quiet

io:::start
/args[0]->b_flags & B_READ/
{
	@io[execname] = sum(args[0]->b_bcount);
}

profile:::tick-5sec
{
  printa("%S:%@d\n", @io);
  printf("\n");
  clear(@io);
}