using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EnteredTunnelAccess
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EnteredTunnelAccess); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EnteredTunnelAccess)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize TunnelAccessId
            s.Write(value.TunnelAccessId);
            //  Serialize EnteringEntityId
            s.Write(value.EnteringEntityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EnteredTunnelAccess)) as Rts.CnC.Messages.Client.EnteredTunnelAccess;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize TunnelAccessId
            s.Read(out value.TunnelAccessId);
            //  Deserialize EnteringEntityId
            s.Read(out value.EnteringEntityId);

            return value;
        }
        
    }
}
