using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PingReply
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PingReply); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PingReply)obj;
            //  Serialize ClientStartTime
            s.Write(value.ClientStartTime);
            //  Serialize ServerTime
            s.Write(value.ServerTime);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PingReply)) as Rts.CnC.Messages.Client.PingReply;
            //  Deserialize ClientStartTime
            s.Read(out value.ClientStartTime);
            //  Deserialize ServerTime
            s.Read(out value.ServerTime);

            return value;
        }
        
    }
}
