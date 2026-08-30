using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestRPC
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestRPC); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestRPC)obj;
            //  Serialize Id
            s.Write(value.Id);
            //  Serialize Arguments
            s.Write(value.Arguments);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestRPC)) as Rts.CnC.Messages.Client.RequestRPC;
            //  Deserialize Id
            s.Read(out value.Id);
            //  Deserialize Arguments
            s.Read(out value.Arguments);

            return value;
        }
        
    }
}
