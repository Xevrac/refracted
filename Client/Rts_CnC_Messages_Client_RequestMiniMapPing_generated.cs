using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestMiniMapPing
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestMiniMapPing); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestMiniMapPing)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Position
            s.Write(value.Position);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestMiniMapPing)) as Rts.CnC.Messages.Client.RequestMiniMapPing;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Position
            s.Read(out value.Position);

            return value;
        }
        
    }
}
