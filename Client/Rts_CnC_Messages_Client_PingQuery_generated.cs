using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PingQuery
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PingQuery); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PingQuery)obj;
            //  Serialize ClientStartTime
            s.Write(value.ClientStartTime);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PingQuery)) as Rts.CnC.Messages.Client.PingQuery;
            //  Deserialize ClientStartTime
            s.Read(out value.ClientStartTime);

            return value;
        }
        
    }
}
